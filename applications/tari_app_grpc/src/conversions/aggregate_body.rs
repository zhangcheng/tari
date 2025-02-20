// Copyright 2020. The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::convert::TryFrom;

use tari_core::transactions::aggregated_body::AggregateBody;
use tari_utilities::convert::try_convert_all;

use crate::tari_rpc as grpc;

impl TryFrom<AggregateBody> for grpc::AggregateBody {
    type Error = String;

    fn try_from(source: AggregateBody) -> Result<Self, Self::Error> {
        let (inputs, outputs, kernels) = source.dissolve();
        Ok(Self {
            inputs: inputs
                .into_iter()
                .map(grpc::TransactionInput::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            outputs: outputs.into_iter().map(grpc::TransactionOutput::from).collect(),
            kernels: kernels.into_iter().map(grpc::TransactionKernel::from).collect(),
        })
    }
}

impl TryFrom<grpc::AggregateBody> for AggregateBody {
    type Error = String;

    fn try_from(body: grpc::AggregateBody) -> Result<Self, Self::Error> {
        let inputs = try_convert_all(body.inputs).map_err(|err: String| format!("inputs {}", err))?;
        let outputs = try_convert_all(body.outputs).map_err(|err: String| format!("outputs {}", err))?;
        let kernels = try_convert_all(body.kernels).map_err(|err: String| format!("kernels {}", err))?;
        let body = AggregateBody::new(inputs, outputs, kernels);
        Ok(body)
    }
}
