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
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { library } from '@fortawesome/fontawesome-svg-core';
import {
  faCaretUp,
  faCaretDown,
  faCheck,
  faPenSquare,
  faPlus,
  faTimes,
  faTrashAlt
} from '@fortawesome/free-solid-svg-icons';

import { ServiceProvider } from './state/service-context';
import FilterBar from './components/FilterBar';
import ProductsTable from './components/ProductsTable';
import { AddProductForm } from './components/AddProductForm';
import { EditProductForm } from './components/EditProductForm';
import './App.scss';

library.add(faCaretUp, faCaretDown, faCheck, faPenSquare, faPlus, faTimes, faTrashAlt);

function App() {
  const initialFormState = {
    formName: '',
    params: {}
  };
  const [activeForm, setActiveForm] = useState(initialFormState);

  function addProduct() {
    setActiveForm({
      formName: 'add-product',
      params: {}
    });
  }

  function editProduct(properties) {
    setActiveForm({
      formName: 'edit-product',
      params: {
        properties
      }
    });
  }

  function openForm(form) {
    const adata = { ...form.params } || {};
    switch (form.formName) {
      case 'add-product':
        return (
          <AddProductForm closeFn={() => setActiveForm(initialFormState)} />
        );
      case 'edit-product':
        return (
          <EditProductForm
            closeFn={() => setActiveForm(initialFormState)}
            properties={adata.properties}
          />
        );
      default:
    }
    return null;
  }

  return (
    <ServiceProvider>
      <div className="product-app">
        <FilterBar />
        <ProductsTable editFn={editProduct} />
        <button className="fab add-product" type="button" onClick={addProduct}>
          <FontAwesomeIcon icon={faPlus} />
        </button>
        {activeForm.formName && openForm(activeForm)}
      </div>
    </ServiceProvider>
  );
}

export default App;
