# REVErse songs

## General

This folder contains many song for reversing some parts of the file
format.

So we have:

 * `DEFAULT.m8s` : old default song
 * `V4EMPTY.m8s` : Empty song in the 4.0 format
 * `V5EMPTY.m8s` : Empty song in the 4.2 format (FW 5.0) (different form V4EMPTY.m8s)
 * `V6EMPTY_beta.m8s` : Empty song of FW 6.0, not analyzed yet
 * `TEST-FILE.m8s` : Historic test song

## Bundle

We have a song and it's bundled counterpart:

 * `FDUB3.m8s` : original song
 * `FDUB3_BUNDLED.m8s` : Song file after bundling.

## CommandMappingV4

So we have a literal test song named `CMDMAPPING.m8s`
that try to list all parameters and instructions along with
screenshot of the various configuration of instruments/phrase/song.

With this it was relatively straightforward to map all the missing
commands. This is expected to be a reference for reverse engineering
the file format.

As much as possible the filename include the instrument/eq number/phrase
for ease of mapping.

 * `CMDMAPPING_4_0.m8s` : first version of the file, saved with a firmware v4
 * `CMDMAPPING_5_0.m8s` : same file as before, but saved with a firmware 5.0.
        There is differences :).
 * `CMDMAPPING_6_0.m8s` : same file as before, but including but saved with a firmware 6.0 beta

