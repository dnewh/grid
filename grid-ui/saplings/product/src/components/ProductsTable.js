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

import React, { useState, useEffect } from 'react';
import PropTypes from 'prop-types';
import { useServiceState } from '../state/service-context';

import './ProductsTable.scss';
import ProductCard from './ProductCard';
import mockProducts from '../test/mock-products';

function ProductsTable({ editFn }) {
  const [products, setProducts] = useState(mockProducts);
  const { selectedService } = useServiceState();

  useEffect(() => {
    if (selectedService === 'all') {
      setProducts(mockProducts);
    } else {
      setProducts(
        mockProducts.filter(product => product.service_id === selectedService)
      );
    }
  }, [selectedService]);

  const productCards = products.map(product => {
    const findProperty = name => {
      const property = product.properties.find(p => p.name === name);
      return property ? property.string_value : null;
    };

    return (
      <ProductCard
        key={`${findProperty('gtin')}_${product.service_id}`}
        gtin={findProperty('gtin')}
        name={findProperty('product_name')}
        owner={product.owner}
        imageURL={findProperty('image_url')}
        editFn={editFn}
        properties={product.properties}
      />
    );
  });

  return (
    <div className="products-table-container">
      <div className="products-table-header">
        <h5 className="title">Products</h5>
        <hr />
      </div>
      <div className="products-table">{productCards}</div>
    </div>
  );
}

ProductsTable.propTypes = {
  editFn: PropTypes.func.isRequired
};

export default ProductsTable;
