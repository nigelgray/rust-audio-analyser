import rust_audio_tester
import kivy
kivy.require('1.11.1')
from kivy.app import App
from kivy.uix.tabbedpanel import TabbedPanel
from kivy.properties import StringProperty
from kivy.uix.textinput import TextInput

class Tabbed(TabbedPanel):
    rmsGain = StringProperty("--- dB")
    generatedTHD = StringProperty("--- %")
    generatedPeakFrequency = StringProperty("--- Hz")
    recordedTHD = StringProperty("--- %")
    recordedPeakFrequency = StringProperty("--- Hz")

    frequency = 1000

    def __init__(self, **kwargs):
        super(Tabbed, self).__init__(**kwargs)

    def doWork(self):
        testFrequency = 1000
        try:
            testFrequency = int(self.frequency)
        except:
            testFrequency = 1000
        rust_audio_tester.set_frequency(testFrequency)
        rust_audio_tester.process_audio()

        self.rmsGain = '{:3.4f} dB'.format(rust_audio_tester.get_rms_gain())
        self.generatedTHD = '{:3.4f} dB'.format(rust_audio_tester.get_generated_thd())
        self.generatedPeakFrequency = '{:6.0f} Hz'.format(rust_audio_tester.get_generated_peak_frequency())
        self.recordedTHD = '{:3.4f} dB'.format(rust_audio_tester.get_recorded_thd())
        self.recordedPeakFrequency = '{:6.0f} Hz'.format(rust_audio_tester.get_recorded_peak_frequency())

    def processFrequencyTest(self, text):
        self.frequency = text

class AudioTesterApp(App):
    def build(self):
        self.title = 'Audio Tester'
        return Tabbed()

if __name__ == '__main__':
    AudioTesterApp().run()
