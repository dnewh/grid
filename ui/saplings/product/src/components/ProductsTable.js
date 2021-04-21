/**
 * Copyright 2018-2021 Cargill Incorporated
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

import React, { useState, useMemo, useEffect } from 'react';
import Icon from '@material-ui/core/Icon';
import xml2js from 'xml2js';
import { matchSorter } from 'match-sorter';
import _ from 'lodash';
import { Chip, Chips } from './Chips';
import { Table } from './Table';
import { useServiceState } from '../state/service-context';
import { Input } from './Input';
import { listProducts } from '../api/grid';
import './ProductsTable.scss';

function ProductsTable() {
  const [products, setProducts] = useState([]);
  const { selectedService } = useServiceState();
  const [filterInputState, setFilterInputState] = useState({
    type: 'product name',
    value: ''
  });
  const initialFilterState = [
    {
      type:'product name',
      value: ''
    },
    {
      type: 'gtin',
      value: ''
    },
    {
      type: 'owner',
      value: ''
    }
  ]
  const [filterState, setFilterState] = useState(initialFilterState);

  const productXMLToJSON = product => {
    let parser = new xml2js.Parser({
      explicitArray: false,
      strict: false,
    });
    parser = parser.parseString;
    const productJSON = [];
    const xmlProperty = _.find(product.properties, ['name', 'xml_data']).string_value;
    parser(xmlProperty, (err, result) => {
      if (err) {
        console.error(err);
      } else {
        productJSON.push(result);
      }
    });
    return productJSON;
  }

  useEffect(() => {
    const getProducts = async () => {
      if (selectedService !== 'none') {
        try {
          const productList = await listProducts(selectedService);
          setProducts(productList.map(p => ({
            product: p,
            data: productXMLToJSON(p)
          })));
        } catch (e) {
          console.error(`Error listing products: ${e}`);
        }
      } else {
        setProducts([]);
      }
    };

    getProducts();
  }, [selectedService]);

  const handleFilterInputChange = e => {
    const { name, value } = e.target;
    switch (name) {
      case 'filter-type':
        setFilterInputState({
          type: value,
          value: filterInputState.value
        });
        break;
      case 'filter-value':
        setFilterInputState({
          type: filterInputState.type,
          value
        });
        break;
      default:
        break;
    }
  };

  const handleAddFilter = () => {
      filterState.find(f => f.type === filterInputState.type).value = filterInputState.value;
      setFilterState([
        ...filterState
      ])

      setFilterInputState({
        type: filterInputState.type,
        value: ''
      })
  }

  const handleRemoveFilter = c => {
    filterState.find(f => f.type === c).value = '';
      setFilterState([
        ...filterState
      ])
  }

  const fuzzyTextFilter = (rows, id, filterValue) => {
    return matchSorter(rows, filterValue, { keys: [row => row.values[id]] });
  }

  const accessProductImage = row => {
    const ns = row.data[0].GRIDTRADEITEMS.TRADEITEM.TRADEITEMINFORMATION.EXTENSION["NS8:REFERENCEDFILEDETAILINFORMATIONMODULE"];
    if (ns) {
      const img = _.find(ns, (r) => r.REFERENCEDFILETYPECODE === "PRODUCT_IMAGE")
      if (img) {
        return img.UNIFORMRESOURCEIDENTIFIER;
      }
    }
    return undefined;
  }

  const accessProductName = row => {
    if (row.data[0].GRIDTRADEITEMS.TRADEITEM.TRADEITEMINFORMATION.EXTENSION["NS9:TRADEITEMDESCRIPTIONMODULE"]) {
      return row.data[0].GRIDTRADEITEMS.TRADEITEM.TRADEITEMINFORMATION.EXTENSION["NS9:TRADEITEMDESCRIPTIONMODULE"].TRADEITEMDESCRIPTIONINFORMATION.REGULATEDPRODUCTNAME._;
    }
    return undefined;
  }

  const filterTypes = useMemo(
    () => ({
      fuzzyText: fuzzyTextFilter,
    })
  )

  const filters = useMemo(
    () => filterState
  )

  const data = useMemo(
    () => products
  )

  const columns = useMemo(
    () => [
      {
        Header: 'GTIN',
        accessor: 'data[0].GRIDTRADEITEMS.TRADEITEM.GTIN',
        id: 'gtin',
        filter: 'fuzzyText'
      },
      {
        Header: 'Image',
        accessor: accessProductImage,
        id: 'image',
        // eslint-disable-next-line react/prop-types, react/destructuring-assignment
        Cell: props => <img src={props.value} alt={`${props.row.values.gtin} thumbnail`} className="product-image" />
      },
      {
        Header: 'Product Name',
        accessor: accessProductName,
        id: 'product name',
        // eslint-disable-next-line react/prop-types, react/destructuring-assignment
        Cell: props => <a href={`/product/${props.row.values.gtin}`}>{props.value}</a>,
        filter: 'fuzzyText'
      },
      {
        Header: 'Created',
        accessor: 'product.created',
      },
      {
        Header: 'Circuit Name/ID',
        accessor: 'product.service_id',
      },
      {
        Header: 'Owner',
        accessor: 'product.orgName',
        filter: 'fuzzyText',
        id: 'owner'
      },
    ]
  )

  return (
    <div className="products-table-container">
      <h1 className="selected-service">{selectedService === 'none' ? 'No service selected' : selectedService}</h1>
      <div className="table-utils">
        <div className="util-wrapper">
          <div className="filters">
            <div className="inputs">
              <Input type="select" icon="filter_list" label="Filter By" name="filter-type" value={filterInputState.type} onChange={handleFilterInputChange}>
                <option value="product name" default>
                  Product name
                </option>
                <option value="gtin">GTIN</option>
                <option value="owner">Owner</option>
              </Input>
              <Input type="text" icon="search" label="Product Name" name="filter-value" value={filterInputState.value} onChange={handleFilterInputChange} />
              <button className="btn-primary" onClick={handleAddFilter} type="button">Add Filter</button>
            </div>
            <div className="filter-list">
              {filterState.some(f => f.value !== '') &&
                <>
                  <Chips>
                    {
                      filterState.map(({type, value}) => (value !== '' && <Chip deleteable key={type} label={`${type}: ${value}`} removeFn={() => handleRemoveFilter(type)} />))
                    }
                  </Chips>
                  <button className="btn-primary btn-min" onClick={() => setFilterState(initialFilterState)} type="button">Clear Filters</button>
                </>
              }
            </div>
          </div>
          <div className="actions">
            <button className="btn-primary" type="button">
              <Icon>add</Icon>
              <span>Add</span>
            </button>
          </div>
        </div>
      </div>
      <div className="table">
        <Table columns={columns} data={data} filterTypes={filterTypes} filters={filters} />
      </div>
    </div>
  );
}

export default ProductsTable;
