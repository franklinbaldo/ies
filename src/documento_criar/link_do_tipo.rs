//! Links de tipo publicados na lista de documentos (#246).
//! O oráculo SEI 4.0.9 repete id_procedimento com o mesmo valor.
//! Conserva a URL assinada original para o GET; só a cópia usada para
//! validação perde a segunda ocorrência idêntica.
use super::ErroCriar;
use crate::arvore_processo::endereco::conferir_endereco_relativo;
use crate::instalacao::Instalacao;

pub(super) fn conferir_link_do_tipo(
    instalacao: &Instalacao,
    href: &str,
    id_procedimento: &str,
) -> Result<String, ErroCriar> {
    let recusado = ErroCriar::EnderecoDoTipoNaoReconhecido;
    let consulta = href.strip_prefix("controlador.php?").ok_or(recusado)?;
    let mut copia = String::from("controlador.php?");
    let mut identidades = 0_u8;
    for parametro in consulta.split('&') {
        let (nome, valor) = parametro.split_once('=').ok_or(recusado)?;
        if nome == "id_procedimento" {
            identidades += 1;
            if identidades > 2 || valor != id_procedimento {
                return Err(recusado);
            }
            // Na captura, as duas identidades são literalmente iguais.
            // O validador compartilhado continua exigindo uma só.
            if identidades == 2 {
                continue;
            }
        }
        if !copia.ends_with('?') {
            copia.push('&');
        }
        copia.push_str(parametro);
    }
    if !(1..=2).contains(&identidades) {
        return Err(recusado);
    }
    conferir_endereco_relativo(
        instalacao,
        &copia,
        "documento_gerar",
        id_procedimento,
    )
    .map_err(|_| recusado)?;
    // Não reconstruir nem reinterpretar a assinatura do link original.
    Ok(format!("{}/{href}", instalacao.url_base()))
}
