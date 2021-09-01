# rufka

Rust implementation of kafka

## Log structure

```
    ${LOG_DIR}\
        logs\
            ${topic.name}\
                ${partition}\
                    ${fmt %020d start_offset}.log{|.index|.cleaned}
        broker\
            raft\
                ...
            config\
                config.yaml


```