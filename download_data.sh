#!/bin/bash


mkdir data
pushd data

# BrightKite dataset

# download and unzip
wget http://snap.stanford.edu/data/loc-brightkite_totalCheckins.txt.gz
gzip -d loc-brightkite_totalCheckins.txt.gz
# extract ids of users with 2100 checkins (the maximum number in the dataset)
cut -f1 loc-brightkite_totalCheckins.txt | uniq -c | sed 's/^ *//' | egrep '^[3-9][0-9][0-9]|^[1-2][0-9][0-9][0-9]' | cut -d' ' -f2 > brightkite_topUsers.txt
# extract data of those users, and prepare datasets for them
for u in `cat brightkite_topUsers.txt`
do
  grep -P '^'$u'\t' loc-brightkite_totalCheckins.txt > full_bk$u.txt
  cut -f 3 full_bk$u.txt > bk$u.txt
done

# remove temporary files
rm full_bk*.txt
rm loc-brightkite_totalCheckins.txt
rm brightkite_topUsers.txt 
