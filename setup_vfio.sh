#!/bin/sh
modprobe vfio_pci
echo 15b3 1017 > /sys/bus/pci/drivers/vfio-pci/new_id
