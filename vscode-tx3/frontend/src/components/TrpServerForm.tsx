import React, { useState, useEffect } from "react";

import Box from "./Box";
import Title from "./Title";
import Form, { FieldType, FormMode } from "./Form";

export interface TrpServerFormProps {
  onUpdate: (trpUrl: string, headers?: Record<string, string>) => void;
}

interface FormData {
  trpEndpoint: string;
  trpUrl: string;
  demeterApiKey: string;
}

const DEMETER_TRP_URL = "https://cardano-preview.trp-m1.demeter.run";
const DEMETER_API_KEY = "trpjodqbmjblunzpbikpcrl";

const TrpServerForm: React.FC<TrpServerFormProps> = (props: TrpServerFormProps) => {
  const [formData, setFormData] = useState<FormData>({
    trpEndpoint: "demeter",
    trpUrl: "",
    demeterApiKey: DEMETER_API_KEY,
  });

  useEffect(() => onUpdate(formData), []);

  const onUpdate = (formData: FormData) => {
    setFormData(formData);
    if (formData.trpEndpoint === "demeter") {
      props.onUpdate(DEMETER_TRP_URL, { "dmtr-api-key": formData.demeterApiKey });
    } else {
      props.onUpdate(formData.trpUrl);
    }
  }

  return (
    <div className="mb-4">
      <Title>TRP Server</Title>
      <Box>
        <Form
          mode={FormMode.Blur}
          onSubmit={data => onUpdate(data as any)}
          fields={[{
            name: "trpEndpoint",
            label: "TRP Endpoint",
            type: FieldType.Select,
            defaultValue: formData.trpEndpoint,
            options: [
              { value: "demeter", label: "Demeter Cardano Preview" },
              { value: "custom", label: "Custom TRP Endpoint" }
            ],
          }, {
            name: "trpUrl",
            label: "TRP Url",
            type: FieldType.Text,
            defaultValue: formData.trpUrl,
            hidden: formData.trpEndpoint === "demeter",
          }, {
            name: "demeterApiKey",
            label: "Demeter API Key",
            type: FieldType.Text,
            defaultValue: formData.demeterApiKey,
            hidden: formData.trpEndpoint !== "demeter",
          }]}
        />
      </Box>
    </div>
  );
}

export default TrpServerForm;