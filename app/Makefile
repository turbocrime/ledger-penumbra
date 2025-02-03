#*******************************************************************************
#  Ledger App
#  (c) 2018 - 2024 Zondax AG
#  (c) 2017 Ledger
#
#  Licensed under the Apache License, Version 2.0 (the "License");
#  you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
#*******************************************************************************

ifeq ($(BOLOS_SDK),)
$(error BOLOS_SDK is not set)
endif

MY_DIR := $(dir $(lastword $(MAKEFILE_LIST)))

# Set the default value for PRODUCTION_BUILD to 0 if not already defined
PRODUCTION_BUILD ?= 1

# Display whether this is a production build or for internal use
ifeq ($(PRODUCTION_BUILD), 1)
    $(info ************ PRODUCTION_BUILD  = [PRODUCTION BUILD])
else
    $(info ************ PRODUCTION_BUILD  = [INTERNAL USE])
endif

# Display whether swap functionality is enabled or not
ifeq ($(ENABLE_SWAP), 1)
    $(info ************ HAVE_SWAP  = [ENABLED])
else
    $(info ************ HAVE_SWAP  = [DISABLED])
endif

# Add the PRODUCTION_BUILD definition to the compiler flags
DEFINES += PRODUCTION_BUILD=$(PRODUCTION_BUILD)

include $(CURDIR)/../deps/ledger-zxlib/makefiles/Makefile.app_testing

ifndef COIN
COIN=UM
endif

VARIANT_PARAM=COIN
VARIANT_VALUES=$(COIN)

include $(CURDIR)/Makefile.version

$(info COIN  = [$(COIN)])

ifeq ($(COIN), UM)
# Main app configuration
DEFINES += APP_STANDARD
APPNAME = "Penumbra"
APPPATH = "44'/6532'"

else
define error_message

COIN value not supported: [$(COIN)]

endef
$(error "$(error_message)")
endif

$(info PATHS LIST = $(APPPATH))

APP_LOAD_PARAMS = --curve secp256k1 $(COMMON_LOAD_PARAMS) --path $(APPPATH)
INCLUDES_PATH += $(CURDIR)/src/protobuf
INCLUDES_PATH += $(CURDIR)/src/common

# Flag to compile all Penumbra actions
FULL_APP ?= 0
ifeq ($(FULL_APP),1)
DEFINES += FULL_APP
$(info ************ FULL APP ENABLED ************)
endif

RUST_TARGET:=thumbv6m-none-eabi

$(info ************ RUST_TARGET  = [$(RUST_TARGET)])

include $(BOLOS_SDK)/Makefile.target

include $(CURDIR)/../deps/ledger-zxlib/makefiles/Makefile.devices

include $(CURDIR)/../deps/ledger-zxlib/makefiles/Makefile.platform

APP_SOURCE_PATH += $(CURDIR)/rust/include
CFLAGS += -Wvla
LDFLAGS += -z muldefs
LDLIBS   += -lm -lgcc -lc
LDLIBS += -L$(MY_DIR)rust/target/$(RUST_TARGET)/release -lrslib

include $(CURDIR)/../deps/ledger-zxlib/makefiles/Makefile.installer_script

$(info TARGET_NAME  = [$(TARGET_NAME)])
$(info ICONNAME  = [$(ICONNAME)])

ifndef ICONNAME
$(error ICONNAME is not set)
endif

ifeq ($(DEBUG), 1)
CFLAGS += -O3 -Os -Wno-unknown-pragmas -Wno-unused-parameter -g
else
CFLAGS += -O3 -Os -Wno-unknown-pragmas -Wno-unused-parameter
endif

# make rust a prerequisite for all object files
$(OBJECT_FILES): rust

.PHONY: rust
rust:
ifeq ($(FULL_APP), 1)
	cd rust && RUSTC_BOOTSTRAP=1 CARGO_HOME="$(CURDIR)/rust/.cargo" cargo build --target $(RUST_TARGET) --release --features full_app
else
	cd rust && RUSTC_BOOTSTRAP=1 CARGO_HOME="$(CURDIR)/rust/.cargo" cargo build --target $(RUST_TARGET) --release
endif

# Before linking, we need to be sure rust lib is there
bin/app.elf: rust

.PHONY: rust_clean
rust_clean:
	cd rust && CARGO_HOME="$(CURDIR)/rust/.cargo" cargo clean

clean: rust_clean

#add dependency on custom makefile filename
dep/%.d: %.c Makefile

.PHONY: version
version:
	@echo "v$(APPVERSION)" > app.version
