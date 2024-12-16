curl -X POST -H "Content-Type: application/json" --data @announce.json "http://0.0.0.0:53317/api/localsend/v2/register"

curl -X POST -H "Content-Type: application/json" --data @example.json "http://0.0.0.0:53317/api/localsend/v2/prepare-upload"

curl -X POST -H "Content-Type: image/jpeg" --data-binary @snowcat.png "http://0.0.0.0:53317/api/localsend/v2/upload?sessionId=c47a926a-cc57-464c-bd2f-406c16baa7c0&fileId=rand&token=fadc734b-24cc-48c9-95ed-4fdf389b1fb5"
