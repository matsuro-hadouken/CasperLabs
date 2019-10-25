%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: casperlabs-engine-grpc-server
Summary: WASM execution engine for CasperLabs smart contracts
Version: @@VERSION@@
Release: 1
License: ASL 2.0
Group: Applications/System
Source0: %{name}-%{version}.tar.gz

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}

%post
# Default Variables
# ---
DEFAULT_USERNAME="casperlabs"
DEFAULT_DATA_DIRECTORY="/var/lib/${DEFAULT_USERNAME}"

# User Creation
# ---
# Assure DEFAULT_USERNAME user exists
if id -u ${DEFAULT_USERNAME} >/dev/null 2>&1; then
    echo "User ${DEFAULT_USERNAME} already exists."
else
    adduser --no-create-home --user-group --system ${DEFAULT_USERNAME}
fi

# Creation of Files/Directories
# ---
# Assure DEFAULT_DATA_DIRECTORY is available for state data
if [ -d ${DEFAULT_DATA_DIRECTORY} ] ; then
    echo "Directory ${DEFAULT_DATA_DIRECTORY} already exists."
else
    mkdir -p ${DEFAULT_DATA_DIRECTORY}
fi

# Files/Directories Owner
# ---
# Assure DEFAULT_DATA_DIRECTORY is owned by DEFAULT_USERNAME
if [ -d ${DEFAULT_DATA_DIRECTORY} ] ; then
    chown ${DEFAULT_USERNAME}:${DEFAULT_USERNAME} ${DEFAULT_DATA_DIRECTORY}
fi

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*
