//! Local OCI image builds with Docker and Buildah.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail, ensure};

use crate::ImageEngine;
use crate::policy;
use crate::process::{CommandSpec, best_effort, output, probe, run as run_command};

const DOCKER_GUIDANCE: &str =
    "Follow https://docs.docker.com/engine/install/ and verify the local daemon is reachable.";
const BUILDAH_GUIDANCE: &str = "Follow https://github.com/containers/buildah/blob/main/install.md and verify its storage/runtime setup.";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImagePlan {
    pub file: &'static str,
    pub context: &'static str,
    pub tag: String,
    pub platform: &'static str,
    pub smoke_program: &'static str,
    pub smoke_args: &'static [&'static str],
}

pub fn run(root: &Path, requested: ImageEngine, tag: String) -> Result<()> {
    let engine = resolve_engine(root, requested)?;
    let plan = plan(tag);
    match engine {
        ImageEngine::Docker => docker(root, &plan),
        ImageEngine::Buildah => buildah(root, &plan),
        ImageEngine::Auto => unreachable!("auto is resolved before execution"),
    }
}

pub fn plan(tag: String) -> ImagePlan {
    ImagePlan {
        file: policy::IMAGE_FILE,
        context: policy::IMAGE_CONTEXT,
        tag,
        platform: policy::IMAGE_PLATFORM,
        smoke_program: policy::IMAGE_SMOKE_PROGRAM,
        smoke_args: policy::IMAGE_SMOKE_ARGS,
    }
}

pub fn resolve_engine(root: &Path, requested: ImageEngine) -> Result<ImageEngine> {
    if requested != ImageEngine::Auto {
        engine_probe(root, requested).map_err(anyhow::Error::msg)?;
        return Ok(requested);
    }

    let mut failures = Vec::new();
    for engine in policy::IMAGE_AUTO_ORDER {
        match engine_probe(root, engine) {
            Ok(()) => return Ok(engine),
            Err(error) => {
                eprintln!("Skipping {engine:?}: {error}");
                failures.push(error);
            }
        }
    }
    bail!(
        "no usable OCI engine in configured auto order:\n- {}",
        failures.join("\n- ")
    )
}

fn engine_probe(root: &Path, engine: ImageEngine) -> Result<(), String> {
    match engine {
        ImageEngine::Docker => probe(
            root,
            "Docker",
            &CommandSpec::new("docker").args(["info", "--format", "{{.ServerVersion}}"]),
            DOCKER_GUIDANCE,
        ),
        ImageEngine::Buildah => probe(
            root,
            "Buildah",
            &CommandSpec::new("buildah").arg("info"),
            BUILDAH_GUIDANCE,
        ),
        ImageEngine::Auto => Err("auto is not an executable engine".to_string()),
    }
}

pub fn docker_build_spec(plan: &ImagePlan) -> CommandSpec {
    CommandSpec::new("docker").args([
        "build",
        "--platform",
        plan.platform,
        "--tag",
        plan.tag.as_str(),
        "--file",
        plan.file,
        plan.context,
    ])
}

pub fn docker_smoke_spec(plan: &ImagePlan, name: &str) -> CommandSpec {
    let mut spec = CommandSpec::new("docker").args([
        "run",
        "--name",
        name,
        "--rm",
        "--network",
        "none",
        "--entrypoint",
        plan.smoke_program,
        plan.tag.as_str(),
    ]);
    spec.args
        .extend(plan.smoke_args.iter().map(|arg| (*arg).into()));
    spec
}

pub fn docker_inspect_spec(plan: &ImagePlan, format: &str) -> CommandSpec {
    CommandSpec::new("docker").args(["image", "inspect", "--format", format, plan.tag.as_str()])
}

pub fn docker_cleanup_spec(name: &str) -> CommandSpec {
    CommandSpec::new("docker").args(["rm", "--force", name])
}

pub fn buildah_build_spec(plan: &ImagePlan) -> CommandSpec {
    CommandSpec::new("buildah").args([
        "bud",
        "--platform",
        plan.platform,
        "--tag",
        plan.tag.as_str(),
        "--file",
        plan.file,
        plan.context,
    ])
}

pub fn buildah_from_spec(plan: &ImagePlan, name: &str) -> CommandSpec {
    CommandSpec::new("buildah").args(["from", "--name", name, plan.tag.as_str()])
}

pub fn buildah_smoke_spec(plan: &ImagePlan, name: &str) -> CommandSpec {
    let mut spec = CommandSpec::new("buildah").args(["run", "--network", "none", name, "--"]);
    spec.args.push(plan.smoke_program.into());
    spec.args
        .extend(plan.smoke_args.iter().map(|arg| (*arg).into()));
    spec
}

pub fn buildah_cleanup_spec(name: &str) -> CommandSpec {
    CommandSpec::new("buildah").args(["rm", name])
}

fn docker(root: &Path, plan: &ImagePlan) -> Result<()> {
    run_command(root, &docker_build_spec(plan))?;
    inspect_image(root, plan, ImageEngine::Docker)?;
    let name = temporary_name();
    let result = smoke_version(root, &docker_smoke_spec(plan, &name));
    best_effort(root, &docker_cleanup_spec(&name));
    result
}

fn buildah(root: &Path, plan: &ImagePlan) -> Result<()> {
    run_command(root, &buildah_build_spec(plan))?;
    inspect_image(root, plan, ImageEngine::Buildah)?;
    let name = temporary_name();
    let result = (|| {
        run_command(root, &buildah_from_spec(plan, &name))?;
        smoke_version(root, &buildah_smoke_spec(plan, &name))
    })();
    best_effort(root, &buildah_cleanup_spec(&name));
    result
}

fn inspect_image(root: &Path, plan: &ImagePlan, engine: ImageEngine) -> Result<()> {
    let (user_spec, entrypoint_spec, os_spec, arch_spec) = match engine {
        ImageEngine::Docker => (
            docker_inspect_spec(plan, "{{.Config.User}}"),
            docker_inspect_spec(plan, "{{json .Config.Entrypoint}}"),
            docker_inspect_spec(plan, "{{.Os}}"),
            docker_inspect_spec(plan, "{{.Architecture}}"),
        ),
        ImageEngine::Buildah => (
            CommandSpec::new("buildah").args([
                "inspect",
                "--type",
                "image",
                "--format",
                "{{.OCIv1.Config.User}}",
                plan.tag.as_str(),
            ]),
            CommandSpec::new("buildah").args([
                "inspect",
                "--type",
                "image",
                "--format",
                "{{json .OCIv1.Config.Entrypoint}}",
                plan.tag.as_str(),
            ]),
            CommandSpec::new("buildah").args([
                "inspect",
                "--type",
                "image",
                "--format",
                "{{.OCIv1.OS}}",
                plan.tag.as_str(),
            ]),
            CommandSpec::new("buildah").args([
                "inspect",
                "--type",
                "image",
                "--format",
                "{{.OCIv1.Architecture}}",
                plan.tag.as_str(),
            ]),
        ),
        ImageEngine::Auto => unreachable!("auto is resolved before execution"),
    };

    let user = inspect_text(root, &user_spec)?;
    ensure!(
        user == "nobody",
        "expected Config.User=nobody, got {user:?}"
    );
    let entrypoint = inspect_text(root, &entrypoint_spec)?;
    ensure!(
        entrypoint.contains("protoc-gen-mdbook"),
        "unexpected Entrypoint: {entrypoint}"
    );
    let os = inspect_text(root, &os_spec)?;
    let arch = inspect_text(root, &arch_spec)?;
    ensure!(
        os == "linux" && arch == "amd64",
        "expected linux/amd64 image, got {os}/{arch}"
    );
    Ok(())
}

fn inspect_text(root: &Path, spec: &CommandSpec) -> Result<String> {
    let output = output(root, spec)?;
    ensure!(
        output.status.success(),
        "image inspect failed ({}): {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn smoke_version(root: &Path, spec: &CommandSpec) -> Result<()> {
    let output = output(root, spec)?;
    ensure!(
        output.status.success(),
        "image --version smoke failed ({}): {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    ensure!(
        stdout.contains("protoc-gen-mdbook"),
        "image --version stdout missing plugin name: {stdout}"
    );
    let pin = protobuf_mdbook::mdbook_version();
    ensure!(
        stdout.contains(pin),
        "image --version stdout missing mdbook pin {pin}: {stdout}"
    );
    eprintln!("xtask: image --version ok: {}", stdout.trim());
    Ok(())
}

fn temporary_name() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    format!("xtask-smoke-{}-{millis}", std::process::id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_order_has_both_concrete_engines_once() {
        assert_eq!(policy::IMAGE_AUTO_ORDER.len(), 2);
        assert!(policy::IMAGE_AUTO_ORDER.contains(&ImageEngine::Docker));
        assert!(policy::IMAGE_AUTO_ORDER.contains(&ImageEngine::Buildah));
        assert_ne!(policy::IMAGE_AUTO_ORDER[0], policy::IMAGE_AUTO_ORDER[1]);
    }

    #[test]
    fn docker_build_is_linux_amd64_scratch_plugin() {
        let plan = plan("protobuf-mdbook:test".to_string());
        let spec = docker_build_spec(&plan);
        assert!(spec.args.iter().any(|arg| arg == "--platform"));
        assert!(spec.args.iter().any(|arg| arg == "linux/amd64"));
        assert!(spec.args.iter().any(|arg| arg == "Dockerfile"));
        assert!(spec.args.iter().any(|arg| arg == "protobuf-mdbook:test"));
    }

    #[test]
    fn docker_smoke_is_offline_and_cleans_up() {
        let plan = plan(policy::IMAGE_TAG.to_string());
        let smoke = docker_smoke_spec(&plan, "xtask-smoke-test");
        assert!(smoke.args.iter().any(|arg| arg == "--network"));
        assert!(smoke.args.iter().any(|arg| arg == "none"));
        assert!(smoke.args.iter().any(|arg| arg == "--rm"));
        assert!(smoke.args.iter().any(|arg| arg == "--version"));
        let cleanup = docker_cleanup_spec("xtask-smoke-test");
        assert!(cleanup.args.iter().any(|arg| arg == "--force"));
    }

    #[test]
    fn buildah_smoke_runs_the_plugin_entrypoint() {
        let plan = plan(policy::IMAGE_TAG.to_string());
        let smoke = buildah_smoke_spec(&plan, "xtask-smoke-test");
        assert!(smoke.args.iter().any(|arg| arg == "--network"));
        assert!(smoke.args.iter().any(|arg| arg == "/protoc-gen-mdbook"));
        assert!(smoke.args.iter().any(|arg| arg == "--version"));
    }

    #[test]
    fn temporary_container_name_is_scoped() {
        assert!(temporary_name().starts_with("xtask-smoke-"));
    }
}
