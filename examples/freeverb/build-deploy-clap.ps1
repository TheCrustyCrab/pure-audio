cargo build
Remove-Item ..\..\target\debug\freeverb.clap
Rename-Item -Path ..\..\target\debug\freeverb.dll -NewName freeverb.clap
Copy-Item -Path ..\..\target\debug\freeverb.clap -Destination 'C:\Program Files\Common Files\CLAP'
Copy-Item -Path ..\..\target\debug\freeverb.pdb -Destination 'C:\Program Files\Common Files\CLAP'