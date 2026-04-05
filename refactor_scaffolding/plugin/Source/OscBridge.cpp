#include "OscBridge.h"

OscBridge::OscBridge(juce::AudioProcessorValueTreeState& vts)
    : valueTree(vts)
{
    // TODO: register as listener for all parameters
    // for (auto* param : valueTree.processor.getParameters())
    //     valueTree.addParameterListener(param->paramID, this);
}

OscBridge::~OscBridge()
{
    stop();
    // TODO: remove all parameter listeners
}

void OscBridge::start()
{
    // TODO: bind receive UDP socket
    // TODO: start receiveLoop on a juce::Thread
}

void OscBridge::stop()
{
    // TODO: signal receiveLoop thread to exit
    // TODO: close UDP sockets
}

void OscBridge::parameterChanged(const juce::String& paramID, float newValue)
{
    // Called on the message thread when DAW automation moves a parameter
    sendOscParamUpdate(paramID, newValue);
}

void OscBridge::sendOscParamUpdate(const juce::String& paramID, float value)
{
    // TODO: encode OSC message /param/set [paramID, value]
    // TODO: send UDP datagram to Bun's receive port
}

void OscBridge::receiveLoop()
{
    // TODO: runs on background thread
    // TODO: recv UDP datagram -> decode OSC
    // TODO: on /param/set: call valueTree.getParameter(id)->setValueNotifyingHost(value)
    // TODO: exit cleanly when stop() signals shutdown
}
