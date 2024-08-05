# Data

These binary files were produced from the `KDMX20220305_233003_V06` data file of an EF4 tornado near Des Moines, IA on 
March 5, 2022. A bespoke script was used to decompress LDM records and isolate individual blocks of encoded data such as
a message header and a digital radar data message body.

## `message_header`

This is a sample of a message header. The header is 28 bytes long and contains information about the message type, the
message length, and the time the message was collected.

## `digital_radar_data_message`

This is a sample of a digital radar data message (type 31). This message contains a variety of operational parameters
and radar data products.