#pragma once

#include <juce_audio_processors/juce_audio_processors.h>
#include "PluginProcessor.h"
#include "WebviewManager.h"

/**
 * PluginEditor
 *
 * Opens a native window containing a WebBrowserComponent pointed at the
 * Bun-served Svelte GUI. In standalone mode the Bun process is spawned here;
 * in plugin mode it is expected to already be running, or is spawned on first
 * editor open and kept alive until the last editor closes.
 */
class PluginEditor : public juce::AudioProcessorEditor
{
public:
    explicit PluginEditor(PluginProcessor&);
    ~PluginEditor() override;

    void resized() override;
    void paint(juce::Graphics&) override;

private:
    PluginProcessor& processor;
    WebviewManager webviewManager;

    // TODO: status overlay shown while Bun is starting up

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(PluginEditor)
};
