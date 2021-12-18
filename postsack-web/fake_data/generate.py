# parse all the json files in this folder and generate a rust file
# containing generated data

import glob
import json
import random
import sys

entries = []

output_rust_file = "../src/generated.rs"

# Coalesce data
for json_file in glob.glob('*.json'):
    parsed = json.load(open(json_file, "r"))
    entries.extend(parsed)

# For each entry, generate a struct

to_address = "john@doe.com"
to_name = ""

struct_template = """Entry { %(fields)s }"""

output_template = """
use super::database::Entry;
pub const ENTRIES: [Entry; %(amount)s] = [
%(content)s
];
"""

# To generate some more data, we keep some email addresses to
# generate 10-12 additional emails with that address afterwards
additional_emails = []

generated_entries = []

def fields_from_entry(entry):
    k = {}
    k["sender_name"] = entry["name"]
    email = entry["email"].split("@")
    k["sender_domain"] = email[1]
    k["sender_local_part"] = email[0]
    date = entry["date"].split(",")
    (k["year"], k["month"], k["day"]) = (int(date[0]), int(date[1]), int(date[2]))
    k["timestamp"] = int(entry["time"])
    k["is_reply"] = True if entry["reply"] == 1 else False
    k["is_send"] = True if entry["send"] == 1 else False
    k["subject"] = entry["subject"]
    k["to_address"] = to_address
    k["to_name"] = to_name
    return k

def fields_to_string(k):
    fields = []
    for key in k:
        value = k[key]
        if type(value) == type(0):
            fields.append("%s: %s" % (key, value))
        elif type(value) == type(True):
            fields.append("%s: %s" % (key, "true" if value == True else "false"))
        elif type(value) == type(""):
            fields.append("%s: \"%s\"" % (key, value))
        elif type(value) == type(u""):
            fields.append("%s: \"%s\"" % (key, value))
        else:
            print(value, type(value))
            sys.exit(0)
    return ", ".join(fields)

# first run over the emails
for entry in entries:
    k = fields_from_entry(entry)

    # Generate additional mails
    if random.uniform(0.0, 1.0) > 0.7:
        for _ in range(0, random.randint(5, 50)):
            additional_emails.append((entry["email"], entry["name"]))

    joined_fields = fields_to_string(k)
    generated_entries.append(struct_template % { "fields": joined_fields })

# second run over the email to generate additional entries with the same
# email address so we have some clusters
for (entry, (email, name)) in zip(entries, additional_emails):
    entry["email"] = email
    entry["name"] = name
    k = fields_from_entry(entry)
    joined_fields = fields_to_string(k)
    generated_entries.append(struct_template % { "fields": joined_fields })

writer = open(output_rust_file, "w")
entries_string = ",\n".join(generated_entries)
writer.write(output_template % { "content": entries_string, "amount": len(generated_entries) })
writer.close()
