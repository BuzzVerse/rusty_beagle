import io, os, sys, time, subprocess, csv
from datetime import datetime

bandwidths = [
        "bandwidth_7_8kHz",
        "bandwidth_10_4kHz",
        "bandwidth_15_6kHz",
        "bandwidth_20_8kHz",
        "bandwidth_31_25kHz",
        "bandwidth_41_7kHz",
        "bandwidth_62_5kHz",
        "bandwidth_125kHz",
        "bandwidth_250kHz",
        "bandwidth_500kHz",
        ]

coding_rates = [
        "coding_4_5",
        "coding_4_6",
        "coding_4_7",
        "coding_4_8",
        ]

spreading_factors = [
        "spreading_factor_128",
        "spreading_factor_256",
        "spreading_factor_512",
        "spreading_factor_1024",
        "spreading_factor_2048",
        "spreading_factor_4096",
        ]

if len(sys.argv) != 2:
    print("Wrong number of arguments")
    sys.exit(-1)

path_to_binary = sys.argv[1];

now = datetime.now()
dt_string = now.strftime("%Y%m%d%H%M%S")
folder_path = "./tmp"
os.makedirs(folder_path, exist_ok=True)
csv_file = os.path.join(folder_path, f"lora_comunication_stats_{dt_string}.csv")
with open(csv_file, mode='w', newline='') as file:
    writer = csv.writer(file)
    
    # Write the header
    writer.writerow(['Bandwidth', 'Coding Rate', 'Spreading Factor', 'Packages Sent', 'Packages Received', 'CRC Errors'])

    for bandwidth in bandwidths:
        for coding_rate in coding_rates:
            for spreading_factor in spreading_factors:
                # call everything needed
                os.system(f"python3 ./make_config.py {bandwidth} {coding_rate} {spreading_factor}")
                rx_process = subprocess.Popen([path_to_binary, "./rx_conf.ron"], stdout=subprocess.PIPE)
                tx_process = subprocess.Popen([path_to_binary, "./tx_conf.ron"], stdout=subprocess.PIPE)

                # wait some time for packages
                time.sleep(0.1) # wait 5 minutes 
                rx_process.terminate()
                tx_process.terminate()

                # collect data
                rx_stdout, rx_stderr = rx_process.communicate()
                tx_stdout, tx_stderr = tx_process.communicate()
                rx_stdout = rx_stdout.decode("utf-8")
                tx_stdout = tx_stdout.decode("utf-8")

                packages_sent = tx_stdout.count("Packet sent.")
                packeges_received = rx_stdout.count("Received")
                crc_errors = rx_stdout.count("CRC Error")
                
                writer.writerow([bandwidth, coding_rate, spreading_factor, packages_sent, packeges_received, crc_errors])

                print(f"Did: {bandwidth}, {coding_rate}, {spreading_factor}\n")
