LowHttpPack
===========

Low-tech http website packer that pack an entire folder and binarise ready-to-send anwser request for an http server like [LowHttpServer](https://github.com/lamogui/LowHttpServer).

How-to-send
-----------

Just run the exe in cmdline using ```./LowHttpPack(.exe) <path/to/website/pack> <optionalNameFor.pack>```. The default name for the .pack is website.pack, if no argument is provided *LowHttpPack* pack it's current working directory. 

Additional informations
-----------------------
 * ***LowHttpPack*** pack all files recursively, be sure to not have any sensitive informations inside packed files. 
 * ***LowHttpPack*** replace all request to ```<parent>/index.html``` by ```<parent>/``` to automatically match the request root file request of the browsers.
 * ***LowHttpPack*** deduce compress files that can be compressed (text and binary if not already compressed format like zip or png) in gzip http compression based on their file extension. It also deduce mimetype for the request from their (because browser often require them).