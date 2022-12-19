use super::helpers::{BaseConstraintBuilder, ColumnTransition};
use crate::{
    constraints,
    mpt_circuit::account_leaf::AccountLeafCols,
    mpt_circuit::branch::BranchCols,
    mpt_circuit::columns::{DenoteCols, PositionCols, ProofTypeCols},
    mpt_circuit::storage_leaf::StorageLeafCols,
    util::Expr,
};
use gadgets::util::{and, not, or, sum};
use halo2_proofs::{arithmetic::FieldExt, plonk::ConstraintSystem, poly::Rotation};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub(crate) struct SelectorsConfig<F> {
    _marker: PhantomData<F>,
}

impl<F: FieldExt> SelectorsConfig<F> {
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        proof_type: ProofTypeCols<F>,
        position_cols: PositionCols<F>,
        branch: BranchCols<F>,
        account_leaf: AccountLeafCols<F>,
        storage_leaf: StorageLeafCols<F>,
        denoter: DenoteCols<F>,
    ) -> Self {
        // It needs to be ensured that:
        // - The selectors denoting the row type are boolean values.
        // - For sets of selectors that are mutually exclusive, it needs to be ensured
        //   that their sum is 1 (for example the selector for the proof type).
        // - The proper order of rows.
        meta.create_gate("Selectors", |meta| {
            let mut cb = BaseConstraintBuilder::default();
            constraints!{[meta, cb], {
            let q_enable = meta.query_fixed(position_cols.q_enable, Rotation::cur());
            let q_not_first = meta.query_fixed(position_cols.q_not_first, Rotation::cur());
            let not_first_level = meta.query_advice(position_cols.not_first_level, Rotation::cur());
            let sel1 = meta.query_advice(denoter.sel1, Rotation::cur());
            let sel2 = meta.query_advice(denoter.sel2, Rotation::cur());

            let is_leaf_s_key = ColumnTransition::new(meta, storage_leaf.is_s_key);
            let is_leaf_s_value = ColumnTransition::new(meta, storage_leaf.is_s_value);
            let is_leaf_c_key = ColumnTransition::new(meta, storage_leaf.is_c_key);
            let is_leaf_c_value = ColumnTransition::new(meta, storage_leaf.is_c_value);
            let is_leaf_in_added_branch =
                ColumnTransition::new(meta, storage_leaf.is_in_added_branch);
            let is_leaf_non_existing = ColumnTransition::new(meta, storage_leaf.is_non_existing);
            let is_non_existing_storage_row =
                ColumnTransition::new(meta, storage_leaf.is_non_existing);
            let is_account_leaf_key_s = ColumnTransition::new(meta, account_leaf.is_key_s);
            let is_account_leaf_key_c = ColumnTransition::new(meta, account_leaf.is_key_c);
            let is_account_leaf_nonce_balance_s =
                ColumnTransition::new(meta, account_leaf.is_nonce_balance_s);
            let is_account_leaf_nonce_balance_c =
                ColumnTransition::new(meta, account_leaf.is_nonce_balance_c);
            let is_account_leaf_storage_codehash_s =
                ColumnTransition::new(meta, account_leaf.is_storage_codehash_s);
            let is_account_leaf_storage_codehash_c =
                ColumnTransition::new(meta, account_leaf.is_storage_codehash_c);
            let is_account_leaf_in_added_branch =
                ColumnTransition::new(meta, account_leaf.is_in_added_branch);
            let is_non_existing_account_row =
                ColumnTransition::new(meta, account_leaf.is_non_existing_account_row);
            let is_extension_node_s = ColumnTransition::new(meta, branch.is_extension_node_s);
            let is_extension_node_c = ColumnTransition::new(meta, branch.is_extension_node_c);
            let is_branch_init = ColumnTransition::new(meta, branch.is_init);
            let is_branch_child = ColumnTransition::new(meta, branch.is_child);
            let is_last_branch_child = ColumnTransition::new(meta, branch.is_last_child);
            let is_modified = ColumnTransition::new(meta, branch.is_modified);
            let is_at_drifted_pos = ColumnTransition::new(meta, branch.is_at_drifted_pos);

            let proof_type_id = ColumnTransition::new(meta, proof_type.proof_type);
            let is_storage_mod = ColumnTransition::new(meta, proof_type.is_storage_mod);
            let is_nonce_mod = ColumnTransition::new(meta, proof_type.is_nonce_mod);
            let is_balance_mod = ColumnTransition::new(meta, proof_type.is_balance_mod);
            let is_codehash_mod = ColumnTransition::new(meta, proof_type.is_codehash_mod);
            let is_account_delete_mod =
                ColumnTransition::new(meta, proof_type.is_account_delete_mod);
            let is_non_existing_account_proof =
                ColumnTransition::new(meta, proof_type.is_non_existing_account_proof);
            let is_non_existing_storage_proof =
                ColumnTransition::new(meta, proof_type.is_non_existing_storage_proof);

            // Row type selectors
            let row_type_selectors = [
                is_branch_init.expr(),
                is_branch_child.expr(),
                is_extension_node_s.expr(),
                is_extension_node_c.expr(),
                is_leaf_s_key.expr(),
                is_leaf_c_key.expr(),
                is_leaf_s_value.expr(),
                is_leaf_c_value.expr(),
                is_leaf_in_added_branch.expr(),
                is_non_existing_storage_row.expr(),
                is_account_leaf_key_s.expr(),
                is_account_leaf_key_c.expr(),
                is_non_existing_account_row.expr(),
                is_account_leaf_nonce_balance_s.expr(),
                is_account_leaf_nonce_balance_c.expr(),
                is_account_leaf_storage_codehash_s.expr(),
                is_account_leaf_storage_codehash_c.expr(),
                is_account_leaf_in_added_branch.expr(),
            ];

            // Proof type selectors
            let proof_type_selectors = [
                is_nonce_mod.expr(),
                is_balance_mod.expr(),
                is_codehash_mod.expr(),
                is_non_existing_account_proof.expr(),
                is_account_delete_mod.expr(),
                is_storage_mod.expr(),
                is_non_existing_storage_proof.expr(),
            ];

            // Sanity checks on all rows
            ifx!{q_enable => {
                // It needs to be ensured that all selectors are boolean. To trigger the
                // constraints for a specific row the selectors could be of any
                // nonnegative value, but being booleans it makes it easier to
                // write the constraints that ensure the subsets of constraints are
                // mutually exclusive and the constraints to ensure the proper order of rows.
                let misc_selectors = vec![
                    not_first_level.expr(),
                    is_last_branch_child.expr(),
                    is_branch_child.expr() * sel1.expr(),
                    is_branch_child.expr() * sel2.expr(),
                    is_modified.expr(),
                    is_at_drifted_pos.expr(),
                ];
                for selector in misc_selectors
                    .iter()
                    .chain(row_type_selectors.iter().chain(proof_type_selectors.iter()))
                {
                    require!(selector => bool);
                }

                // The type of the row needs to be set (if all selectors would be 0 for a row,
                // then all constraints would be switched off).
                require!(sum::expr(row_type_selectors.iter()) => 1);

                // The type of the proof needs to be set.
                require!(sum::expr(proof_type_selectors.iter()) => 1);

                // We need to prevent lookups into non-lookup rows and we need to prevent for
                // example nonce lookup into balance lookup row.
                let proof_type_lookup_row_types = [
                    is_account_leaf_nonce_balance_s.expr(),
                    is_account_leaf_nonce_balance_c.expr(),
                    is_account_leaf_storage_codehash_c.expr(),
                    is_non_existing_account_row.expr(),
                    is_account_leaf_key_s.expr(),
                    is_leaf_c_value.expr(),
                    is_non_existing_storage_row.expr(),
                ];
                for (idx, (proof_type, row_type)) in proof_type_selectors
                    .iter()
                    .zip(proof_type_lookup_row_types.iter())
                    .enumerate()
                {
                    // Proof type is 0 everywhere except in the specific lookup row
                    require!(proof_type_id.expr() * proof_type.expr() * (row_type.expr() - 1.expr()) => 0);

                    ifx!{proof_type_id.expr(), row_type.expr() => {
                        // Proof type is the expected value on the specific lookup row type
                        require!(proof_type_id => idx + 1);
                    }}
                }
            }};

            // First row
            ifx!{q_enable, not::expr(q_not_first.expr()) => {
                // In the first row only account leaf key S row or branch init row can occur
                require!(or::expr([is_account_leaf_key_s.cur(), is_branch_init.cur()]) => true);
            }};

            // All rows except the first row
            ifx!{q_not_first => {
                // State transitions
                let transitions = [
                    // Branch init can start:
                    // - after another branch (means after extension node C)
                    // - after account leaf (account -> storage proof)
                    // - after storage leaf (after storage mod proof ends)
                    // - in the first row
                    (
                        "Last branch row/last storage leaf/last account leaf -> branch init",
                        1.expr(),
                        vec![
                            is_extension_node_c.prev(),
                            is_account_leaf_in_added_branch.prev(),
                            is_leaf_non_existing.prev(),
                        ],
                        is_branch_init.cur(),
                    ),
                    // Account leaf key S can appear after
                    // - extension node C (last branch row)
                    // - the last storage leaf row (if only one account in the trie) and there is
                    //   another proof above it (having last storage leaf row as the last row)
                    (
                        "Last branch row/last storage leaf -> account leaf key S",
                        is_account_leaf_key_s.cur(),
                        vec![is_extension_node_c.prev(), is_leaf_non_existing.prev()],
                        is_account_leaf_key_s.cur(),
                    ),
                    // Storage leaf key S can appear after
                    // - extension node C (last branch row)
                    // - account leaf storage codehash C
                    (
                        "Last branch row/last storage leaf -> account leaf key S",
                        is_leaf_s_key.cur(),
                        vec![
                            is_extension_node_c.prev(),
                            is_account_leaf_in_added_branch.prev(),
                        ],
                        is_leaf_s_key.cur(),
                    ),
                    (
                        "Last branch row -> extension node S",
                        1.expr(),
                        vec![is_last_branch_child.prev()],
                        is_extension_node_s.cur(),
                    ),
                    (
                        "Extension node S -> extension node C",
                        1.expr(),
                        vec![is_extension_node_s.prev()],
                        is_extension_node_c.cur(),
                    ),
                    (
                        "Account leaf key S -> account leaf key C",
                        1.expr(),
                        vec![is_account_leaf_key_s.prev()],
                        is_account_leaf_key_c.cur(),
                    ),
                    (
                        "Account leaf key C -> non existing account row",
                        1.expr(),
                        vec![is_account_leaf_key_c.prev()],
                        is_non_existing_account_row.cur(),
                    ),
                    (
                        "Non existing account row -> account leaf nonce balance S",
                        1.expr(),
                        vec![is_non_existing_account_row.prev()],
                        is_account_leaf_nonce_balance_s.cur(),
                    ),
                    (
                        "Account leaf nonce balance S -> account leaf nonce balance C",
                        1.expr(),
                        vec![is_account_leaf_nonce_balance_s.prev()],
                        is_account_leaf_nonce_balance_c.cur(),
                    ),
                    (
                        "Account leaf nonce balance C -> account leaf storage codehash S",
                        1.expr(),
                        vec![is_account_leaf_nonce_balance_c.prev()],
                        is_account_leaf_storage_codehash_s.cur(),
                    ),
                    (
                        "Account leaf storage codehash S -> account leaf storage codehash C",
                        1.expr(),
                        vec![is_account_leaf_storage_codehash_s.prev()],
                        is_account_leaf_storage_codehash_c.cur(),
                    ),
                    (
                        "Account leaf storage codehash C -> account leaf added in branch",
                        1.expr(),
                        vec![is_account_leaf_storage_codehash_c.prev()],
                        is_account_leaf_in_added_branch.cur(),
                    ),
                    (
                        "Storage leaf key S -> storage leaf value S",
                        1.expr(),
                        vec![is_leaf_s_key.prev()],
                        is_leaf_s_value.cur(),
                    ),
                    (
                        "Storage leaf value S -> storage leaf key C",
                        1.expr(),
                        vec![is_leaf_s_value.prev()],
                        is_leaf_c_key.cur(),
                    ),
                    (
                        "Storage leaf key C -> storage leaf value C",
                        1.expr(),
                        vec![is_leaf_c_key.prev()],
                        is_leaf_c_value.cur(),
                    ),
                    (
                        "Storage leaf value C -> storage leaf in added branch",
                        1.expr(),
                        vec![is_leaf_c_value.prev()],
                        is_leaf_in_added_branch.cur(),
                    ),
                    (
                        "Storage leaf in added branch -> storage leaf non existing row",
                        1.expr(),
                        vec![is_leaf_in_added_branch.prev()],
                        is_leaf_non_existing.cur(),
                    ),
                ];
                for (name, condition, from, to) in transitions {
                    ifx!{condition => {
                        require!(name, to => {from});
                    }}
                }

                // Data transitions
                // Note that these constraints do not prevent attacks like putting account leaf
                // rows more than once - however, doing this would lead into failure
                // of the constraints responsible for address RLC (or key RLC if storage rows).
                // Also, these constraints do not guarantee there is an account proof before
                // storage proof - constraints for this are implemented using address_rlc column
                // to be changed to the proper value only in the account leaf key row.
                let modifications = [
                    ("is_nonce_mod", is_nonce_mod),
                    ("is_balance_mod", is_balance_mod),
                    ("is_codehash_mod", is_codehash_mod),
                    (
                        "is_non_existing_account_proof",
                        is_non_existing_account_proof,
                    ),
                    ("is_account_delete_mod", is_account_delete_mod),
                    ("is_storage_mod", is_storage_mod),
                    (
                        "is_non_existing_storage_proof",
                        is_non_existing_storage_proof,
                    ),
                ];
                for (name, data) in modifications {
                    // Does not change outside first level
                    ifx!{not_first_level => {
                        require!(name, data.prev() => data.cur());
                    } elsex {
                        // Does not change inside first level except in the first row
                        ifx!{not::expr(is_branch_init.expr()), not::expr(is_account_leaf_key_s.expr()) => {
                            require!(name, data.prev() => data.cur());
                        }}
                    }};
                }
            }}
            }}

            // Internal branch selectors (`is_init`, `is_last_child`) are checked in
            // `branch.rs`.

            cb.gate(1.expr())
        });

        SelectorsConfig {
            _marker: PhantomData,
        }
    }
}
