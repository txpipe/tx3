import React, { useState, useEffect } from "react";

import Box from "./Box";
import Title from "./Title";
import Form, { FieldType, FormMode } from "./Form";

export interface TrpServerFormProps {
  onUpdate: (trpData: TrpServer) => void;
  trpServers: TrpServer[];
}

interface FormData {
  trpServer: string;
}

function getValueFromTrpServer(server: TrpServer) {
  return JSON.stringify(server);
}

function fromValueToTrpServer(server: string): TrpServer | null {
  try {
    return JSON.parse(server);
  } catch {}
  return null;
}

function SettingsButton() {
  return (
    <button
      type="button"
      className="cursor-pointer"
      onClick={() => { vscode.postMessage({ event: 'open-settings', dest: 'tx3.trpServers' }) }}
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.25" viewBox="0 0 24 24">
        <path d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 0 0 2.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 0 0 1.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 0 0-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 0 0-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 0 0-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 0 0-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 0 0 1.066-2.573c-.94-1.543.826-3.31 2.37-2.37 1 .608 2.296.07 2.572-1.065z"/>
        <path d="M9 12a3 3 0 1 0 6 0 3 3 0 0 0-6 0"/>
      </svg>
    </button>
  )
}

const TrpServerForm: React.FC<TrpServerFormProps> = (props: TrpServerFormProps) => {
  const [formData, setFormData] = useState<FormData>({
    trpServer: getValueFromTrpServer(props.trpServers[0]),
  });

  useEffect(() => {
    const server = fromValueToTrpServer(formData.trpServer);
    if (server) {
      props.onUpdate(server)
    }
  }, [formData]);

  const options = props.trpServers.map(server => ({
    label: server.name,
    value: getValueFromTrpServer(server),
  }));

  return (
    <div className="mb-4">
      <Title>TRP Server</Title>
      <Box>
        <Form
          mode={FormMode.Blur}
          onSubmit={data => setFormData(data as any)}
          fields={[{
            name: "trpServer",
            label: null,
            type: FieldType.Select,
            defaultValue: formData.trpServer,
            options,
            suffix: <SettingsButton />,
          }]}
        />
      </Box>
    </div>
  );
}

export default TrpServerForm;