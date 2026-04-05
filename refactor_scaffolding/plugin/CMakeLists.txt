cmake_minimum_required(VERSION 3.22)
project(AudioDspPlugin VERSION 0.1.0)

# TODO: set JUCE_PATH to your local JUCE checkout or use FetchContent
# add_subdirectory(${JUCE_PATH} JUCE)

# TODO: link against pre-built dsp-core static library
# set(DSP_CORE_LIB ../dsp-core/target/release/libdsp_core.a)

juce_add_plugin(AudioDspPlugin
    COMPANY_NAME "YourName"
    IS_SYNTH FALSE
    NEEDS_MIDI_INPUT FALSE
    NEEDS_MIDI_OUTPUT FALSE
    IS_MIDI_EFFECT FALSE
    EDITOR_WANTS_KEYBOARD_FOCUS FALSE
    PLUGIN_MANUFACTURER_CODE Mnfr
    PLUGIN_CODE Adsp
    FORMATS VST3 AU Standalone
    PRODUCT_NAME "Audio DSP Plugin"
)

target_sources(AudioDspPlugin PRIVATE
    Source/PluginProcessor.cpp
    Source/PluginEditor.cpp
    Source/WebviewManager.cpp
    Source/OscBridge.cpp
)

target_compile_features(AudioDspPlugin PRIVATE cxx_std_17)

target_link_libraries(AudioDspPlugin
    PRIVATE
        juce::juce_audio_utils
        juce::juce_gui_extra   # for WebBrowserComponent
        # ${DSP_CORE_LIB}      # TODO: uncomment when ready
    PUBLIC
        juce::juce_recommended_config_flags
        juce::juce_recommended_warning_flags
)
