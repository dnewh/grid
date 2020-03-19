/**
 * Copyright 2018-2020 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
import React, { useState } from 'react';
import PropTypes from 'prop-types';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faTimes } from '@fortawesome/free-solid-svg-icons';
import _ from 'lodash';
import papa from 'papaparse';
import { useServiceState } from '../state/service-context';
import { MultiStepForm, Step, StepInput } from './MultiStepForm';
import { Chips, Chip } from './Chips';
import { MultiSelect } from './MultiSelect';

import './forms.scss';

export function AddProductForm({ closeFn }) {
  const { services } = useServiceState();
  const [selectedServices, setSelectedServices] = useState([]);
  const [imgFile, setImgFile] = useState(null);
  const [imgPreview, setImgPreview] = useState(null);
  const [fileLabel, setFileLabel] = useState('Upload master data file');
  const [imgLabel, setImgLabel] = useState('Upload product image');
  const [attributes, setAttributes] = useState([]);
  const [attrState, setAttrState] = useState({
    type: '',
    name: '',
    value: ''
  });

  const handleFileUpload = e => {
    setFileLabel(e.target.files[0].name);
    papa.parse(e.target.files[0], {
      complete(results) {
        setAttributes([...attributes, ...results.data]);
      },
      header: true,
      skipEmptyLines: true
    });
  };

  const handleImgUpload = e => {
    e.preventDefault();
    const file = e.target.files[0];
    setImgLabel(file.name);

    const reader = new FileReader();
    reader.onloadend = () => {
      setImgFile(file);
      setImgPreview(reader.result);
    };

    reader.readAsDataURL(file);
    return imgFile;
  };

  const handleServiceChange = newServices => {
    setSelectedServices(newServices);
  };

  const addAttribute = e => {
    e.preventDefault();
    setAttributes([...attributes, attrState]);
    setAttrState({
      type: '',
      name: '',
      value: ''
    });
  };

  const removeAttr = attr => {
    setAttributes(attributes.filter(attribute => !_.isEqual(attribute, attr)));
  };

  const handleAttrChange = e => {
    const { name, value } = e.target;
    setAttrState({
      ...attrState,
      [name]: value
    });
  };

  const createAttrData = attribute => {
    return (
      <div className="attribute-data">
        <span className="name">{attribute.name}</span>
        <span className="type">{attribute.type}</span>
        <span className="value">{attribute.value}</span>
      </div>
    );
  };

  const clearState = () => {
    setSelectedServices([]);
    setAttributes([]);
    setAttrState({
      type: '',
      name: '',
      value: ''
    });
    setImgFile(null);
    setImgPreview(null);
    setFileLabel('Upload master data file');
    setImgLabel('Upload product image');
  };

  const submitFn = () => {
    clearState();
  };

  const makeListItems = () => {
    return services.map(service => ({
      label: service.serviceID,
      value: service.id
    }));
  };

  return (
    <div className="modalForm">
      <FontAwesomeIcon icon={faTimes} className="close" onClick={closeFn} />
      <div className="content">
        <MultiStepForm
          formName="Add Product"
          handleSubmit={submitFn}
          disabled={!selectedServices.length && !attributes.length}
        >
          <Step step={1} label="Select service">
            <h6>Select service(s)</h6>
            <MultiSelect
              listItems={makeListItems()}
              placeholder="No services selected"
              onChange={handleServiceChange}
            />
          </Step>
          <Step step={2} label="Add master data">
            <StepInput
              type="file"
              accept="text/csv"
              id="add-master-data-file"
              label={fileLabel}
              onChange={handleFileUpload}
            />
            <div className="divider" />
            <h6>Add attributes</h6>
            <div className="form-group">
              <StepInput
                type="select"
                label="Attribute type"
                name="type"
                value={attrState.type}
                onChange={handleAttrChange}
              >
                <option value="">(none)</option>
                <option value="text" default>
                  Text
                </option>
                <option value="number">Number</option>
                <option value="boolean">Boolean</option>
              </StepInput>
            </div>
            <div className="form-group">
              <StepInput
                type="text"
                label="Attribute name"
                name="name"
                value={attrState.name}
                onChange={handleAttrChange}
              />
              <StepInput
                type="text"
                label="Attribute value"
                name="value"
                value={attrState.value}
                onChange={handleAttrChange}
              />
              <button
                className="confirm"
                type="button"
                onClick={addAttribute}
                disabled={
                  !(attrState.type && attrState.name && attrState.value)
                }
              >
                Add
              </button>
            </div>
            <Chips>
              {attributes.map(attribute => {
                const data = createAttrData(attribute);
                return (
                  <Chip
                    label={attribute.name}
                    data={data}
                    removeFn={() => removeAttr(attribute)}
                    deleteable
                  />
                );
              })}
            </Chips>
          </Step>
          <Step step={3} label="Add attachments">
            <h6>Add additional info</h6>
            <StepInput
              type="file"
              accept="image/png, image/jpeg"
              id="add-master-data-file"
              label={imgLabel}
              onChange={handleImgUpload}
            />
            {imgPreview && (
              <div className="preview-container">
                <img className="img-preview" src={imgPreview} alt="preview" />
              </div>
            )}
          </Step>
          <Step step={4} label="Review and submit">
            <h6>Review new product</h6>
            {imgPreview && (
              <div className="preview-container">
                <img className="img-preview" src={imgPreview} alt="preview" />
              </div>
            )}
            <h6>Selected services</h6>
            <Chips>
              {selectedServices.length > 0 &&
                selectedServices.map(service => {
                  const data = services.filter(s => s.id === service);

                  return data.length && <Chip label={data[0].serviceID} />;
                })}
              {!selectedServices.length && <span>No services selected</span>}
            </Chips>
            <h6>Attributes</h6>
            <Chips>
              {attributes.length > 0 &&
                attributes.map(attribute => {
                  const data = createAttrData(attribute);
                  return <Chip label={attribute.name} data={data} />;
                })}
              {!attributes.length && <span>No attributes entered</span>}
            </Chips>
          </Step>
        </MultiStepForm>
      </div>
    </div>
  );
}

AddProductForm.propTypes = {
  closeFn: PropTypes.func.isRequired
};
