#include "WebviewManager.h"

WebviewManager::WebviewManager()
{
    // TODO: construct WebBrowserComponent with appropriate options
    //   juce::WebBrowserComponent::Options{}
    //     .withKeepPageLoadedWhenBrowserIsHidden()
    browser = std::make_unique<juce::WebBrowserComponent>();
}

WebviewManager::~WebviewManager()
{
    stop();
}

void WebviewManager::start()
{
    spawnBun();
    waitForReady();
    navigateToGui();
}

void WebviewManager::stop()
{
    if (bunProcess && bunProcess->isRunning())
    {
        // TODO: send SIGTERM, wait briefly, then kill
        bunProcess->kill();
        bunProcess = nullptr;
    }

    ready = false;
}

bool WebviewManager::isReady() const
{
    return ready;
}

juce::WebBrowserComponent& WebviewManager::getBrowserComponent()
{
    return *browser;
}

void WebviewManager::spawnBun()
{
    // TODO: resolve path to bun binary (bundle it with the plugin or find on PATH)
    // TODO: resolve path to gui/host.ts relative to plugin bundle
    // TODO: launch: bun run host.ts
    bunProcess = std::make_unique<juce::ChildProcess>();

    juce::StringArray args;
    args.add("bun");
    args.add("run");
    args.add("host.ts"); // TODO: absolute path

    // TODO: bunProcess->start(args)
    // TODO: capture stdout/stderr for logging and ready detection
}

void WebviewManager::waitForReady()
{
    // TODO: read stdout lines from bunProcess in a loop
    // TODO: break when a line contains "READY" or timeout after N seconds
    // TODO: on timeout, show an error in the webview
    pollForReady();
}

void WebviewManager::navigateToGui()
{
    const auto url = juce::URL("http://127.0.0.1:" + juce::String(GUI_PORT));
    browser->goToURL(url.toString(false));
    ready = true;
}

void WebviewManager::pollForReady()
{
    // TODO: temporary naive implementation — wait a fixed delay
    // Replace with proper stdout parsing
    juce::Thread::sleep(1500);
}
