cargo build
Remove-Item ..\..\target\debug\oscillator.clap
Rename-Item -Path ..\..\target\debug\oscillator.dll -NewName oscillator.clap
Copy-Item -Path ..\..\target\debug\oscillator.clap -Destination 'C:\Program Files\Common Files\CLAP'
Copy-Item -Path ..\..\target\debug\oscillator.pdb -Destination 'C:\Program Files\Common Files\CLAP'