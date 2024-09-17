import os
import sys
import sysconfig


def find_pymermaider_bin() -> str:
    """Return the pymermaider binary path."""
    pymermaider_exe = "pymermaider" + sysconfig.get_config_var("EXE")

    scripts_path = os.path.join(sysconfig.get_path("scripts"), pymermaider_exe)
    if os.path.isfile(scripts_path):
        return scripts_path

    if sys.version_info >= (3, 10):
        user_scheme = sysconfig.get_preferred_scheme("user")
    elif os.name == "nt":
        user_scheme = "nt_user"
    elif sys.platform == "darwin" and sys._framework:
        user_scheme = "osx_framework_user"
    else:
        user_scheme = "posix_user"

    user_path = os.path.join(
        sysconfig.get_path("scripts", scheme=user_scheme),
        pymermaider_exe,
    )
    if os.path.isfile(user_path):
        return user_path

    # Search in `bin` adjacent to package root (as created by `pip install --target`).
    pkg_root = os.path.dirname(os.path.dirname(__file__))
    target_path = os.path.join(pkg_root, "bin", pymermaider_exe)
    if os.path.isfile(target_path):
        return target_path

    raise FileNotFoundError(scripts_path)


if __name__ == "__main__":
    pymermaider = os.fsdecode(find_pymermaider_bin())
    if sys.platform == "win32":
        import subprocess

        completed_process = subprocess.run([pymermaider, *sys.argv[1:]])
        sys.exit(completed_process.returncode)
    else:
        os.execvp(pymermaider, [pymermaider, *sys.argv[1:]])
