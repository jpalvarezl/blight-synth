#pragma once

#include <juce_audio_processors/juce_audio_processors.h>

/**
 * OscBridge (JUCE side)
 *
 * Sits between the JUCE AudioProcessorValueTreeState and the Bun OSC layer.
 * When the DAW moves an automatable parameter, this class encodes an OSC
 * message and sends it to Bun so the GUI stays in sync.
 * Conversely, when the user moves a knob in the GUI (OSC comes in via Bun),
 * this class updates the ValueTreeState so the DAW sees the change.
 *
 * In the standalone app this class is not needed — Bun speaks OSC directly
 * with the Rust core. It is only relevant in the plugin context.
 */
class OscBridge : private juce::AudioProcessorValueTreeState::Listener
{
public:
    explicit OscBridge(juce::AudioProcessorValueTreeState& vts);
    ~OscBridge() override;

    void start();
    void stop();

private:
    juce::AudioProcessorValueTreeState& valueTree;

    // TODO: UDP socket for sending OSC to Bun (port 9002 or similar)
    // TODO: UDP socket for receiving OSC from Bun

    /// Called by JUCE when a DAW automation gesture changes a parameter.
    void parameterChanged(const juce::String& paramID, float newValue) override;

    void sendOscParamUpdate(const juce::String& paramID, float value);
    void receiveLoop(); // TODO: run on a background thread

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(OscBridge)
};
