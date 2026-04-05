#include "PluginProcessor.h"
#include "PluginEditor.h"

PluginProcessor::PluginProcessor()
    : AudioProcessor(BusesProperties()
          .withOutput("Output", juce::AudioChannelSet::stereo(), true)),
      parameters(*this, nullptr, "Parameters", createParameterLayout())
{
    // TODO: initialise Rust DSP core via FFI
    // TODO: initialise OscBridge
}

PluginProcessor::~PluginProcessor()
{
    // TODO: shut down Rust DSP core
    // TODO: shut down OscBridge
}

juce::AudioProcessorValueTreeState::ParameterLayout
PluginProcessor::createParameterLayout()
{
    std::vector<std::unique_ptr<juce::RangedAudioParameter>> params;

    // TODO: add parameters matching OSC address space
    // e.g. params.push_back(std::make_unique<juce::AudioParameterFloat>(
    //     "gain", "Gain", 0.0f, 2.0f, 1.0f));

    return { params.begin(), params.end() };
}

void PluginProcessor::prepareToPlay(double sampleRate, int samplesPerBlock)
{
    // TODO: forward sampleRate + blockSize to Rust DSP core via FFI
}

void PluginProcessor::releaseResources()
{
    // TODO: notify Rust core to release audio resources
}

void PluginProcessor::processBlock(juce::AudioBuffer<float>& buffer,
                                   juce::MidiBuffer& midiMessages)
{
    // TODO: read current parameter values from ValueTreeState
    // TODO: forward buffer pointers to Rust DSP core via FFI
    // TODO: (or let Rust own audio I/O entirely and just pass-through here)
}

juce::AudioProcessorEditor* PluginProcessor::createEditor()
{
    return new PluginEditor(*this);
}

void PluginProcessor::getStateInformation(juce::MemoryBlock& destData)
{
    // TODO: serialise ValueTreeState to XML -> destData
}

void PluginProcessor::setStateInformation(const void* data, int sizeInBytes)
{
    // TODO: deserialise XML -> restore ValueTreeState
    // TODO: forward restored params to Rust core
}

int  PluginProcessor::getNumPrograms()                                  { return 1; }
int  PluginProcessor::getCurrentProgram()                               { return 0; }
void PluginProcessor::setCurrentProgram(int)                            {}
const juce::String PluginProcessor::getProgramName(int)                 { return {}; }
void PluginProcessor::changeProgramName(int, const juce::String&)       {}

juce::AudioProcessor* JUCE_CALLTYPE createPluginFilter()
{
    return new PluginProcessor();
}
