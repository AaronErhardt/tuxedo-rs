project(
  'tux-tailord',
  'rust',
  version: '0.1.0',
  meson_version: '>= 0.61',
  license: 'GPLv2',
)

cargo = find_program('cargo', required: true)

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
bin = bindir / meson.project_name()

if get_option('profile') == 'development'
  profile = 'Devel'
else
  profile = ''
endif

systemd = dependency('systemd')
systemd_system_unit_dir = systemd.get_variable(
    pkgconfig: 'systemdsystemunitdir')

global_conf = configuration_data()
global_conf.set('BIN', bin)
service = configure_file(
    input: 'tailord.service.in',
    output: 'tailord.service',
    configuration: global_conf,
    install: true,
    install_dir: systemd_system_unit_dir,
)

dbus_config_dir = '/usr/share/dbus-1/system.d'
install_data('com.tux.Tailor.conf', install_dir: dbus_config_dir)

# Copy default configurations
install_subdir('default_configs/keyboard', install_dir: '/etc/tailord')
install_subdir('default_configs/fan', install_dir: '/etc/tailord')
install_subdir('default_configs/profiles', install_dir: '/etc/tailord')
install_symlink('active_profile.json', install_dir: '/etc/tailord/profiles', pointing_to: 'default.json')

meson.add_install_script('post_install.sh', systemd_system_unit_dir, bindir, dbus_config_dir)

subdir('src')