import rust_audio_tester

rust_audio_tester.set_frequency(15000)

rust_audio_tester.process_audio()

print('Python: RMS Gain =                   ' + '{:.2f}'.format(rust_audio_tester.get_rms_gain()) + ' dB')
print('Python: Generated THD =              ' + '{:.4f}'.format(rust_audio_tester.get_generated_thd()) + ' %')
print('Python: Generated Peak Frequency =   ' + '{:.0f}'.format(rust_audio_tester.get_generated_peak_frequency()) + ' Hz')
print('Python: Recorded THD =               ' + '{:.4f}'.format(rust_audio_tester.get_recorded_thd()) + ' %')
print('Python: Recorded Peak Frequency =    ' + '{:.0f}'.format(rust_audio_tester.get_recorded_peak_frequency()) + ' Hz')
