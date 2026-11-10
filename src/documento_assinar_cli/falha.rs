//! Projeções públicas das assinaturas e da falha do envio reconciliada.
use crate::arvore_processo::Assinante;
use crate::documento_assinar::FalhaDoEnvio;

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

pub(super) fn falha_do_envio_publica(falha: &FalhaDoEnvio) -> crate::saida::FalhaPublica {
    crate::saida::FalhaPublica::nova(falha.status_http())
}
