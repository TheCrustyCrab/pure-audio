cargo build
Remove-Item ..\..\target\debug\surge_synth_saw_demo.clap
Rename-Item -Path ..\..\target\debug\surge_synth_saw_demo.dll -NewName surge_synth_saw_demo.clap
Copy-Item -Path ..\..\target\debug\surge_synth_saw_demo.clap -Destination 'C:\Program Files\Common Files\CLAP'
Copy-Item -Path ..\..\target\debug\surge_synth_saw_demo.pdb -Destination 'C:\Program Files\Common Files\CLAP'