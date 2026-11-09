//! Projecao HTML de uma unica pagina de blocos a partir de estado SQLite.
//! A tela nao e um HTML gravado: faixas e linhas sao consultadas a cada GET.
use crate::{Estado, Resposta, escapar, parametro_unico, protocolo_sintetico};
use iesei::blocos::{self, Pagina, Tipo};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

const HASH: &str = protocolo_sintetico::INFRA_HASH;

pub(crate) fn acrescentar_menu(
    mut resposta: Resposta,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if resposta.estado != Estado::Ok || !resposta.corpo.contains("</body>") {
        return resposta;
    }
    let mut links = String::new();
    for tipo in [Tipo::Assinatura, Tipo::Interno, Tipo::Reuniao] {
        match blocos::ler(banco, instalacao, cenario, tipo) {
            Ok(Some(_)) => {
                write!(links, "<a href='controlador.php?acao={}&amp;infra_hash={HASH}'>Blocos {}</a>",
                    tipo.acao(), tipo.nome()).expect("escrever em String");
            }
            Ok(None) => {}
            Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
        }
    }
    resposta.corpo = resposta
        .corpo
        .replacen("</body>", &format!("{links}</body>"), 1);
    resposta
}

fn html(pagina: &Pagina) -> String {
    let mut saida = String::from("<html><body><table id='tblBlocos'>");
    if let Some((inicio, fim)) = pagina.faixa {
        write!(saida, "<caption>Lista de Blocos ({} registros - {inicio} a {fim}):</caption>",
            pagina.total).expect("escrever em String");
    } else {
        write!(saida, "<caption>Lista de Blocos ({} registros):</caption>",
            pagina.total).expect("escrever em String");
    }
    saida.push_str("<tr><th></th><th>Número</th><th>Estado</th><th>Descrição</th></tr>");
    for linha in &pagina.linhas {
        let numero = escapar(&linha.numero);
        write!(saida, "<tr><td><input type='checkbox'></td><td><a href='controlador.php?acao=rel_bloco_protocolo_listar&amp;id_bloco={numero}&amp;infra_hash={HASH}'>{numero}</a></td><td>{}</td><td>{}</td></tr>",
            escapar(&linha.estado), escapar(&linha.descricao)).expect("escrever em String");
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
    let Some(tipo) = parametro_unico(alvo, "acao").and_then(Tipo::da_acao) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    match blocos::ler(banco, instalacao, cenario, tipo) {
        Ok(Some(pagina)) => Resposta::html(html(&pagina)),
        Ok(None) | Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}
