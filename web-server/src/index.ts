import express from "express";
import compress from "compression";
import pino from "pino";
import path from "path";
import fs from "fs";
import {duration} from "moment";


const HOST = "0.0.0.0";
const PORT = 8080;
const RETRY_INTERVAL = duration({second: 5});

export const logger = pino({
    transport: {
        target: "pino-pretty"
    },
})

export const PROJECT_ROOT_PATH = path.dirname(require.main?.path as string);

const app = express();
app.use(compress());
app.use(express.static("public"));

import {build} from "./buildUnity";

if (require.main === module) {

    (async () => {
        if (!process.env.AI_SERVER_HOST) {
            logger.error("Environment variable of 'AI_SERVER_HOST' is not set");
            process.exit(1);
        }

        if (!fs.existsSync(path.join("public", "Build"))) {
            if (process.env.DOCKER) {
                logger.error("Build not found. Run 'ts-node src/buildUnity.ts' in your local machine with unity installed");
                process.exit(1);
            } else {
                await build();
            }
        }

        while (true) {
            try {
                await fetch(process.env.AI_SERVER_HOST as string, { method: "HEAD" });
                break;
            } catch (e) {
                logger.debug(`Couldn't get a response from ${process.env.AI_SERVER_HOST}. Retrying in ${RETRY_INTERVAL.asSeconds()} seconds...`)
                await new Promise(r => setTimeout(r, RETRY_INTERVAL.asMilliseconds()));
            }
        }

        app.listen(PORT, HOST, () => {
            logger.info(`Server is running on port: ${PORT}`);
        });
    })();
}
