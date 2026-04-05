#pragma once

#include <juce_gui_extra/juce_gui_extra.h>

/**
 * WebviewManager
 *
 * Owns two things:
 *   1. The JUCE WebBrowserComponent (native OS webview) that renders the GUI.
 *   2. The Bun child process that serves the Svelte app and speaks OSC.
 *
 * In the plugin context there is no external network — everything stays on
 * localhost. The webview is pointed at http://127.0.0.1:GUI_PORT once Bun
 * signals it is ready.
 */
class WebviewManager
{
public:
    WebviewManager();
    ~WebviewManager();

    /// Spawn Bun and navigate the webview once ready.
    void start();

    /// Kill Bun and clear the webview.
    void stop();

    bool isReady() const;

    juce::WebBrowserComponent& getBrowserComponent();

private:
    std::unique_ptr<juce::WebBrowserComponent> browser;
    std::unique_ptr<juce::ChildProcess>        bunProcess;

    bool ready { false };

    static constexpr int GUI_PORT = 5173; // Vite default dev port

    void spawnBun();
    void waitForReady();
    void navigateToGui();

    // TODO: replace polling with stdout line parsing for "READY" signal
    void pollForReady();
};
