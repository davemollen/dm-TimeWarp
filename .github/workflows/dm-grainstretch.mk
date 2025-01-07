######################################
#
# dm-grainstretch
#
######################################

DM_GRAINSTRETCH_VERSION = <SHA>
DM_GRAINSTRETCH_SITE = https://github.com/davemollen/dm-GrainStretch.git
DM_GRAINSTRETCH_SITE_METHOD = git
DM_GRAINSTRETCH_BUNDLES = dm-GrainStretch.lv2

define DM_GRAINSTRETCH_BUILD_CMDS
	~/.cargo/bin/rustup default nightly

	rm -f $(@D)/lv2/dm-GrainStretch.lv2/libdm_grain_stretch.so
	(cd $(@D)/lv2 && \
		~/.cargo/bin/cargo build $(MOD_PLUGIN_BUILDER_RUST_BUILD_FLAGS))

	~/.cargo/bin/rustup default stable
endef

define DM_GRAINSTRETCH_INSTALL_TARGET_CMDS
	$(INSTALL) -d $(TARGET_DIR)/usr/lib/lv2
	cp -rv $(@D)/lv2/dm-GrainStretch.lv2 $(TARGET_DIR)/usr/lib/lv2/
	$(INSTALL) -m 644 $(@D)/lv2/target/$(MOD_PLUGIN_BUILDER_RUST_TARGET)/release/libdm_grain_stretch.so $(TARGET_DIR)/usr/lib/lv2/dm-GrainStretch.lv2/
endef

$(eval $(generic-package))
