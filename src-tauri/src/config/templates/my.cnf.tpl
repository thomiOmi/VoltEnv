[mysqld]
basedir = {{ bin_dir }}
datadir = {{ data_dir }}
port = {{ port }}
socket = {{ data_dir }}/mysql.sock
log-error = {{ log_dir }}/mysql-error.log
skip-grant-tables
