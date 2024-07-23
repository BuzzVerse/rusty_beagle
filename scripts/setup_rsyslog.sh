#!/bin/env bash

rusty_log="/etc/rsyslog.d/rusty_beagle.conf"

sudo touch $rusty_log
echo "template(name=\"rusty_beagle_list\" type=\"list\") {
    property(name=\"timereported\" dateFormat=\"rfc3339\")
    constant(value=" ")
    property(name=\"hostname\")
    constant(value=\" \")
    property(name=\"app-name\")
    constant(value=\" [\")
    property(name=\"syslogseverity-text\")
    constant(value=\"]\")
    constant(value=\":\")
    property(name=\"msg\" spifno1stsp=\"on\" )
    property(name=\"msg\" droplastlf=\"on\" )   
    constant(value=\"\n\")
}

if \$programname == 'rusty_beagle' then /var/log/rusty_beagle.log;rusty_beagle_list
& stop" | sudo tee $rusty_log
sudo systemctl restart rsyslog
