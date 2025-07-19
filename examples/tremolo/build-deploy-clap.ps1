cargo build
Remove-Item ..\..\target\debug\tremolo.clap
Rename-Item -Path ..\..\target\debug\tremolo.dll -NewName tremolo.clap
Copy-Item -Path ..\..\target\debug\tremolo.clap -Destination 'C:\Program Files\Common Files\CLAP'
Copy-Item -Path ..\..\target\debug\tremolo.pdb -Destination 'C:\Program Files\Common Files\CLAP'