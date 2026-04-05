#include "PluginEditor.h"

static constexpr int DEFAULT_WIDTH  = 900;
static constexpr int DEFAULT_HEIGHT = 600;

PluginEditor::PluginEditor(PluginProcessor& p)
    : AudioProcessorEditor(p),
      processor(p),
      webviewManager()
{
    setSize(DEFAULT_WIDTH, DEFAULT_HEIGHT);
    setResizable(true, true);

    // TODO: show a loading overlay until webviewManager signals ready

    webviewManager.start();
    addAndMakeVisible(webviewManager.getBrowserComponent());
}

PluginEditor::~PluginEditor()
{
    // TODO: decide lifecycle policy:
    //   - keep Bun alive if other editor instances exist
    //   - shut down only when the last editor closes
    webviewManager.stop();
}

void PluginEditor::resized()
{
    webviewManager.getBrowserComponent().setBounds(getLocalBounds());
}

void PluginEditor::paint(juce::Graphics& g)
{
    // Background is covered by the webview; only visible during loading
    g.fillAll(juce::Colours::black);
}
