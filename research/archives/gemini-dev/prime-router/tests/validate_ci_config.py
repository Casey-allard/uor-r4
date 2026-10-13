from pathlib import Path

import yaml


WORKFLOW_PATH = Path(__file__).resolve().parents[1] / ".github" / "workflows" / "ci.yml"


def require(cond: bool, msg: str):
    if not cond:
        raise AssertionError(msg)


def main():
    require(WORKFLOW_PATH.exists(), f"Workflow not found: {WORKFLOW_PATH}")

    text = WORKFLOW_PATH.read_text(encoding="utf-8")
    data = yaml.load(text, Loader=yaml.BaseLoader)
    require(isinstance(data, dict), "Workflow root must be a YAML mapping")

    require("name" in data and str(data["name"]).strip(), "Workflow must define a name")
    require("on" in data, "Workflow must define triggers")

    triggers = data["on"]

    require(isinstance(triggers, (dict, list, str)), "Workflow triggers must be valid YAML type")

    jobs = data.get("jobs")
    require(isinstance(jobs, dict) and jobs, "Workflow must contain at least one job")
    require("test" in jobs, "Workflow must define a 'test' job")

    test_job = jobs["test"]
    require("runs-on" in test_job, "Test job must define runs-on")
    steps = test_job.get("steps", [])
    require(isinstance(steps, list) and steps, "Test job must define steps")

    step_lines = [str(step.get("run", "")) for step in steps if isinstance(step, dict)]
    merged = "\n".join(step_lines)

    require("python -m unittest -v" in merged, "Workflow must run unit tests")
    require("python tests/validate_ci_config.py" in merged, "Workflow must self-validate CI config")
    require("python -m py_compile" in merged, "Workflow must run compile checks")

    print("CI workflow validation passed.")


if __name__ == "__main__":
    main()
