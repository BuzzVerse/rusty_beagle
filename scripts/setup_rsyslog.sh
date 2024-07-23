#!/bin/env bash

rusty_log="/etc/rsyslog.d/rusty_beagle.conf"

sudo touch $rusty_log
echo "if \$programname == 'rusty_beagle' then /var/log/rusty_beagle.log" | sudo tee $rusty_log
echo "& stop" | sudo tee -a $rusty_log
sudo systemctl restart rsyslog
