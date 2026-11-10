//! Tela de detalhe de bloco gerada do SQLite, nao de HTML gravado.
use crate::{Estado, Resposta, escapar, parametro_unico};
use iesei::bloco_detalhe::{self, Detalhe};
use iesei::blocos::{self, Tipo};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

fn html(d: &Detalhe) -> String {
    let mut saida = String::from("<html><body><table id='tblProtocolosBlocos'>");
    if let Some((inicio, fim)) = d.faixa {
        write!(
            saida,
            "<caption>Lista de Processos/Documentos ({} registros - {inicio} a {fim}):</caption>",
            d.total
        )
        .expect("escrever em String");
    } else {
        write!(
            saida,
            "<caption>Lista de Processos/Documentos ({} registros):</caption>",
            d.total
        )
        .expect("escrever em String");
    }
    saida.push_str(
        "<tr><th></th><th>Documento</th><th>Processo</th><th>Anotações</th><th>Tipo</th></tr>",
    );
    for i in &d.itens {
        write!(
            saida,
            "<tr><td><input type='checkbox'></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            escapar(i.documento.as_deref().unwrap_or("")),
            escapar(&i.processo),
            escapar(&i.anotacoes),
            escapar(i.tipo.as_deref().unwrap_or(""))
        )
        .expect("escrever em String");
    }
    saida.push_str("</table></body></html>");
    saida
}

pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(numero) = parametro_unico(alvo, "id_bloco") else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if numero.is_empty() || !numero.bytes().all(|b| b.is_ascii_digit()) {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    // O link do detalhe so foi publicado na lista de assinatura observada.
    let publicado = blocos::ler(banco, instalacao, cenario, Tipo::Assinatura);
    match publicado {
        Ok(Some(pagina)) if pagina.linhas.iter().any(|l| l.numero == numero) => {}
        _ => return Resposta::nova(Estado::DadosInsuficientes),
    }
    match bloco_detalhe::ler(banco, instalacao, cenario, Tipo::Assinatura, numero) {
        Ok(Some(detalhe)) => Resposta::html(html(&detalhe)),
        _ => Resposta::nova(Estado::DadosInsuficientes),
    }
}
