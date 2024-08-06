import io, os, sys, time, subprocess, csv
from datetime import datetime

if len(sys.argv) != 3:
    print("Wrong number of arguments")
    print("Correct arguments:")
    print("python3 ./stress_test.py [time_to_run] [path_to_binary]")
    sys.exit(-1)

path_to_binary = sys.argv[2];
time_to_run = sys.argv[1];

now = datetime.now()
dt_string = now.strftime("%Y%m%d%H%M%S")
script_dir = os.path.dirname(os.path.abspath(sys.argv[0]))
folder_path = os.path.join(script_dir, "tmp")
os.makedirs(folder_path, exist_ok=True)
csv_file = os.path.join(folder_path, f"lora_comunication_stats_{dt_string}.csv")
with open(csv_file, mode='w', newline='') as file:
    writer = csv.writer(file)
    
    # Write the header
    writer.writerow(['Packages Sent', 'Packages Received', 'CRC Errors', 'Time (s)'])

    # call everything needed
    rx_process = subprocess.Popen([path_to_binary, "./beagle_configs/rx_conf.ron"], stdout=subprocess.PIPE)
    tx_process = subprocess.Popen([path_to_binary, "./beagle_configs/tx_conf.ron"], stdout=subprocess.PIPE)

    # wait some time for packages
    time.sleep(float(time_to_run)) 
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

    writer.writerow([packages_sent, packeges_received, crc_errors, time_to_run])

