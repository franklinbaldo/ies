//! Identidade lexical publicada no Location de cadastro.
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::formulario_web::{Redirecionamento302, consulta_do_controlador};

/// O 302 é somente evidência de identidade, nunca uma ordem de navegação.
/// A validação literal simétrica evita ler parâmetros de um fragmento.
pub(super) fn id_publicado(redir: &Redirecionamento302) -> Option<&str> {
    let local = redir.localizacao()?;
    if local.contains(['#', '\\']) || !local.bytes().all(|b| (33..=126).contains(&b)) {
        return None;
    }
    let consulta =
        ConsultaDoControlador::ler(consulta_do_controlador(local, redir.url_do_envio())?).ok()?;
    if consulta.acao().ok()? != "grupo_unidade_listar" {
        return None;
    }
    let id = consulta.unico("id_grupo").ok()?;
    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then_some(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    const POST: &str = "https://example.invalid/sei/controlador.php?acao=grupo_unidade_cadastrar";

    #[test]
    fn identidade_lexical_e_controlador() {
        for local in [
            "controlador.php?acao=grupo_unidade_listar&id_grupo=001",
            "https://example.invalid/sei/controlador.php?acao=grupo_unidade_listar&id_grupo=001",
            "http://example.invalid/sei/controlador.php?acao=grupo_unidade_listar&id_grupo=001",
        ] {
            let r = Redirecionamento302::sintetico(Some(local), POST);
            assert_eq!(id_publicado(&r), Some("001"));
        }
    }

    #[test]
    fn identidade_nao_publicada_ou_ambigua() {
        for local in [
            None,
            Some("controlador.php?acao=grupo_unidade_listar&x=y#fragment&id_grupo=001"),
            Some("controlador.php?x=y#fragment&acao=grupo_unidade_listar&id_grupo=001"),
            Some("controlador.php?acao=grupo_unidade_listar&id_grupo=001&x=y#fragment"),
            Some("controlador.php?acao=grupo_unidade_listar&id_grupo=001&id_grupo=002"),
            Some("controlador.php?acao=grupo_unidade_listar&id_grupo_unidade=001"),
            Some("controlador.php?acao=grupo_unidade_consultar&id_grupo=001"),
            Some("controlador.php?acao=grupo_unidade_listar&id_grupo="),
            Some("controlador.php?acao=grupo_unidade_listar&id_grupo=%30%31"),
            Some(
                "https://other.invalid/sei/controlador.php?acao=grupo_unidade_listar&id_grupo=001",
            ),
            Some(
                "https://example.invalid/outra/controlador.php?acao=grupo_unidade_listar&id_grupo=001",
            ),
        ] {
            let r = Redirecionamento302::sintetico(local, POST);
            assert_eq!(id_publicado(&r), None);
        }
    }
}
