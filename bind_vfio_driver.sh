#!/bin/sh
PCI_ADDR="$1"
echo "${PCI_ADDR}" > "/sys/bus/pci/devices/${PCI_ADDR}/driver/unbind"
echo "${PCI_ADDR}" > "/sys/bus/pci/drivers/vfio-pci/bind"