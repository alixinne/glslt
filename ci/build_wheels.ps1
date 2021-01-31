$ErrorActionPreference = 'stop'
Set-PSDebug -Trace 1
$RustTarget = $args[0]

Function Test-CommandExists {
	Param ($command)
	Try {
		If (Get-Command $command) {
			Return $true
		}
	} Catch {
		Return $false
	}
}

# Install maturin
if (-Not (Test-CommandExists maturin)) {
	pip install --user -U maturin
}

# Get full path to maturin
$Maturin = "$(python -m site --user-site)\\..\\Scripts\\maturin"

# Install pyenv
if (-Not (Test-CommandExists pyenv)) {
	choco install pyenv-win --version 2.64.3
}

# Refresh environment
refreshenv

# Setup pyenv
$PyEnv = "pyenv"
$env:PATH = "$env:PATH;$env:USERPROFILE\.pyenv\pyenv-win\bin;$env:USERPROFILE\.pyenv\pyenv-win\shims"

# TODO: Support i686
$PyVersionSuffix = ""

# List of Python versions to build for
$PyVersions = "3.5.4", "3.6.8", "3.7.7", "3.8.2"
ForEach ($PyVersion in $PyVersions) {
	# Compute short version name
	$ShortPyVersion = $PyVersion.Split(".")
	$ShortPyVersion = $ShortPyVersion[0] + $ShortPyVersion[1]

	# Install and select Python version
	& $PyEnv install -q "$PyVersion$PyVersionSuffix"
	& $PyEnv global "$PyVersion$PyVersionSuffix"
	& $PyEnv rehash

	# Get Python interp path
	$Python = & $PyEnv which python

	cd glslt
	& $Maturin build --strip "--cargo-extra-args=--features python" --target $RustTarget --release -i $Python
	cd ..
}
