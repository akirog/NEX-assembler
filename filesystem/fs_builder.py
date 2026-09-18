import argparse
from argparse import Namespace

from compiler.compiler import main as compiler_main

# This program will take some number of input files and create a binary file with the binaries and a file table

# It has 3 sections
# KERNEL
# Always stored at 0x00
KERNEL_ADDRESS = 0x00


# File table
# Always stored at 0x10000
# File count at 0x10000
# Actual headers at 0x10004
FILE_TABLE_ADDRESS = 0x10000
FILE_TABLE_SIZE = 0x1000
# Each file header will be formatted as:
"""
    0x00-17: file name          24 bytes
    0x18-1B: file size          4 bytes
    0x1C-1F: binary address     4 bytes 
    very little data for now, if i need more data per header just expand this
"""
FILE_NAME_SIZE = 24
FILE_SIZE_SIZE = 4
FILE_BIN_ADDR_SIZE = 4
FILE_HEADER_SIZE = FILE_NAME_SIZE + FILE_SIZE_SIZE + FILE_BIN_ADDR_SIZE


# Binary section
# This can be dynamically placed since file headers will point here anyway
BINARY_SECTION_ADDRESS = FILE_TABLE_ADDRESS + FILE_TABLE_SIZE




class FsBuilder:
    def __init__(self):
        self.kernel_filepath: str | None = None

        self.verbose = False

        # File name to filepath
        self.files: dict[str, str] = {}


    def build(self):
        # Collect filepaths from config file
        self.collect_filepaths()

        # Load and place files
        binary: bytearray = self.generate_binary()

        # Place binary in a file
        with open("filesystem/fs.bin", "wb") as f:
            f.write(binary)

        pass


    def generate_binary(self) -> bytearray:
        """Takes the filepaths and generates the binary file"""
        binary = bytearray(BINARY_SECTION_ADDRESS)

        if self.kernel_filepath is not None:
            kernel_bin = open(f"{self.kernel_filepath}", "rb").read()

            binary[KERNEL_ADDRESS:FILE_TABLE_ADDRESS] = kernel_bin
            pass

        header_address = FILE_TABLE_ADDRESS
        binary_address = BINARY_SECTION_ADDRESS

        count_size = 4
        binary[header_address:header_address + count_size] = len(self.files).to_bytes(4, byteorder="little")
        header_address += count_size

        for file_name, path in self.files.items():
            # Place the binary
            file_bytes: bytes

            with open(path, "rb") as f:
                file_bytes = f.read()

            ensure_size(binary, binary_address + len(file_bytes))

            binary[binary_address:binary_address + len(file_bytes)] = file_bytes



            # Add the header
            # File name
            name_bytes = file_name.encode()

            if len(name_bytes) > FILE_NAME_SIZE:
                raise ValueError(f"Filename too long: {file_name}")

            name_bytes = name_bytes.ljust(FILE_NAME_SIZE, b'\x00')

            binary[header_address:header_address + FILE_NAME_SIZE] = name_bytes
            header_address += FILE_NAME_SIZE


            # File size
            binary[header_address:header_address + FILE_SIZE_SIZE] = len(file_bytes).to_bytes(FILE_SIZE_SIZE, byteorder="little")
            header_address += FILE_SIZE_SIZE


            # File address
            binary[header_address:header_address+FILE_BIN_ADDR_SIZE] = binary_address.to_bytes(4, byteorder="little")
            header_address += FILE_BIN_ADDR_SIZE


            # Update binary address
            binary_address += len(file_bytes)

            if header_address > BINARY_SECTION_ADDRESS:
                raise RuntimeError(f"File table overflow, most likely an error since this would need {(BINARY_SECTION_ADDRESS-FILE_TABLE_ADDRESS - 4) // FILE_HEADER_SIZE} files")

        return binary



    def collect_filepaths(self):
        """Collects the filepaths given in the config file"""
        lines: list[str] = []

        with open("filesystem/conf.conf", "r") as f:
            lines = f.readlines()


        curr_section = ""

        for line in lines:
            if line.strip() == "":
                continue

            if line.startswith("["):
                # Header thing
                curr_section = line.strip()[1:-1].upper()
                continue



            if curr_section == "":
                print(f"Warning: line skipped since no header set yet: {line}")
                continue

            elif curr_section == "KERNEL":
                if self.kernel_filepath is not None:
                    print(f"Warning: kernel filepath seems to have been overwritten, check if kernel filepath is set multiple times")

                path = line.strip().strip('"').strip("'")
                extension = path.split(".")[-1]
                if extension == "bin":
                    self.kernel_filepath = path

                elif extension == "nex":
                    # Compile it with the nex assembler first
                    output_path = path.removesuffix(".nex") + ".bin"

                    args = Namespace()
                    args.__setattr__("input", path)
                    args.__setattr__("output", output_path)
                    args.__setattr__("verbose", self.verbose)

                    compiler_main(args)

                    self.kernel_filepath = output_path


            elif curr_section == "PROGRAMS":
                parts = line.strip().split(" ")

                path = parts[2].strip('"').strip("'")
                extension = path.split(".")[-1]
                if extension == "bin":
                    self.files[parts[0]] = path

                elif extension == "nex":
                    # Compile it with the nex assembler first
                    output_path = path.removesuffix(".nex") + ".bin"

                    args = Namespace()
                    args.__setattr__("input", path)
                    args.__setattr__("output", output_path)
                    args.__setattr__("verbose", self.verbose)

                    compiler_main(args)

                    self.files[parts[0]] = output_path

            elif curr_section == "FILES":
                # For files just place the direct file
                parts = line.strip().split(" ")
                path = parts[2].strip('"').strip("'")

                self.files[parts[0]] = path


        print(f"KERNEL filepath: {self.kernel_filepath}")
        print()
        print(f"PROGRAM filepaths:")

        for file_name, path in self.files.items():
            print(f"\t{file_name}: {path}")



def ensure_size(buf: bytearray, size: int):
    if len(buf) < size:
        buf.extend(b'\x00' * (size - len(buf)))


if __name__ == "__main__":
    arg_parser = argparse.ArgumentParser(description="Filesystem builder")
    arg_parser.add_argument("--verbose", action="store_true", help="Print debug output")

    args = arg_parser.parse_args()

    builder = FsBuilder()
    builder.verbose = args.verbose
    builder.build()