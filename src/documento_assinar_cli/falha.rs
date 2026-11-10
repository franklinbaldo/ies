//! Projeções públicas das assinaturas e da falha do envio reconciliada.
use crate::arvore_processo::Assinante;

#[derive(serde::Serialize)]
pub(super) struct AssinantePublico<'a> {
    cargo: &'a str,
    nome: &'a str,
    unidade: &'a str,
}

pub(super) fn assinaturas_publicas(lista: &[Assinante]) -> Vec<AssinantePublico<'_>> {
    lista
        .iter()
        .map(|a| AssinantePublico {
            cargo: &a.cargo,
            nome: &a.nome,
            unidade: &a.unidade,
        })
        .collect()
}
