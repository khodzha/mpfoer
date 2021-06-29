## Mpfoer

Discord bot to convert videos to x264 for embedding them into discord chats.


### Build image:
```bash
docker build -t khodzha/mpfoer:latest .
```

### Run image:

Create a file `token.txt` with bot token, then run:

```bash
docker run -v /path/to/token.txt:/app/token.txt -d --restart unless-stopped mpfoer
```
