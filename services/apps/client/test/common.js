// token_type: "hawk", scope: "u|core:cruds sc#apps:rs sc#metrics:c payment#products:rs payment#purchases:crud simcustm#pack:s simcustm#packfile:r payment#transactions:cr payment#prices:s payment#options:s", expires_in: 604800, kid: "FGFYvY+/4XwTYIX9nVi+sXj5tPA=", mac_key: "p7cI80SwX+gmX0G+T938agWAV1eR9wrpCR9JgsoIIlk=", mac_algorithm: "hmac-sha-256" }
//["Hawk id=\"FGFYvY+/4XwTYIX9nVi+sXj5tPA=\"", "ts=\"1611717940\"", "nonce=\"SrnmiS6u9dckTg==\"", "mac=\"gVH14LHIxSTD/Oq7+MsFCpxHzafWRDSEvXlGFnpQAzM=\"", "hash=\"\""]
var token = { keyId: "FGFYvY+/4XwTYIX9nVi+sXj5tPA=", macKey: "p7cI80SwX+gmX0G+T938agWAV1eR9wrpCR9JgsoIIlk=" };

class TokenProvider extends lib_apps.TokenProviderBase {
    constructor(service, session) {
        super(service.id, session);
    }

    display() {
        return "TokenProvider";
    }

    getToken(tokenType) {
        console.log('TokenProvider getToken() is called');

        return Promise.resolve({
            keyId: token.keyId,
            macKey: token.macKey,
            tokenType: tokenType,
        });
    }
}

async function wait_service_state(service, AppsServiceState) {
    while (true) {
        let state = await service.getState();
        if (state == AppsServiceState.RUNNING) {
            break;
        }
        // Wait one second.
        let delay = new Promise(resolve => { window.setTimeout(resolve, 1000); });
        await delay;
    }
}
