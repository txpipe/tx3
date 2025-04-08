import React, { useState, useEffect } from "react";

import Box from "./Box";
import Title from "./Title";
import Input from "./Input";

export interface TrpServerFormProps {
  onUpdate: (trpUrl: string, headers?: Record<string, string>) => void;
}

const TrpServerForm: React.FC<TrpServerFormProps> = (props: TrpServerFormProps) => {
  const demeterTrpUrl = "https://cardano-preview.trp-m1.demeter.run";

  const [trpEndpoint, setTrpEndpoint] = useState<string>("demeter");
  const [trpUrl, setTrpUrl] = useState<string>(demeterTrpUrl);
  const [demeterApiKey, setDemeterApiKey] = useState<string>("trpjodqbmjblunzpbikpcrl");

  useEffect(() => onUpdate(), []);

  const handleTrpEndpointChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setTrpEndpoint(e.target.value);
    if (e.target.value === "demeter") {
      setTrpUrl(demeterTrpUrl);
    }
    onUpdate();
  }

  const handleTrpUrlChange = (value: string) => {
    setTrpUrl(value);
    onUpdate();
  }

  const handleDemeterApiKeyChange = (value: string) => {
    setDemeterApiKey(value);
    onUpdate();
  }

  const onUpdate = () => {
    const headers = trpEndpoint === "demeter" ? { "dmtr-api-key": demeterApiKey } : undefined;
    props.onUpdate(trpUrl, headers);
  }

  return (
    <div className="mb-4">
      <Title>TRP Server</Title>
      <Box>
        <p className="label">TRP Endpoint</p>
        <select
          className="form-input input"
          value={trpEndpoint}
          onChange={handleTrpEndpointChange}
        >
          <option value="demeter">Demeter Cardano Preview</option>
          <option value="custom">Custom TRP Endpoint</option>
        </select>

        <Input
          label="TRP Url"
          value={trpUrl}
          onChange={handleTrpUrlChange}
          disabled={trpEndpoint === "demeter"}
        />
        
        {trpEndpoint === "demeter" &&
          <Input
            label="Demeter API Key"
            value={demeterApiKey}
            onChange={handleDemeterApiKeyChange}
          />
        }
      </Box>
    </div>
  );
}

export default TrpServerForm;