//! A tela do Nível de detalhe do anti-IES reflete somente o SQLite da
//! instalação/cenário. A gravação depende de outra fatia, não há POST fictício.
use crate::{Estado, Resposta, escapar, protocolo_sintetico};
use iesei::colunas_detalhe::{self, Configuracao};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

fn html(configuracao: &Configuracao) -> String {
    let hash = protocolo_sintetico::INFRA_HASH;
    let mut pagina = format!(
        "<html><head><title>SEI - Configurar Detalhe</title></head><body>\
         <form id='frmAcessoExternoCancelar' method='post' onsubmit='return OnSubmitForm();' \
         action='controlador.php?acao=procedimento_configurar_detalhe&amp;acao_origem=procedimento_configurar_detalhe&amp;infra_hash={hash}'>\
         <input type='hidden' id='hdnInfraTipoPagina' name='hdnInfraTipoPagina' value='2'>\
         <button type='submit' id='sbmSalvar' name='sbmSalvar' value='Salvar'>Salvar</button>"
    );
    for opcao in &configuracao.opcoes {
        let campo = opcao.coluna.campo();
        let rotulo = escapar(opcao.coluna.rotulo());
        let checked = if opcao.marcada { " checked='checked'" } else { "" };
        write!(
            pagina,
            "<div><input type='checkbox' id='{campo}' name='{campo}' class='infraCheckbox'{checked}>\
             <label for='{campo}'>{rotulo}</label></div>"
        )
        .expect("String");
    }
    pagina.push_str("</form></body></html>");
    pagina
}

/// Consulta uma observação; sem estado retorna 503, não uma tela vazia.
pub(crate) fn responder(
    metodo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    match colunas_detalhe::ler(banco, instalacao, cenario) {
        Ok(Some(configuracao)) => Resposta::html(html(&configuracao)),
        Ok(None) | Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iesei::colunas_detalhe::{Coluna, Opcao};
    use iesei::observacao::UnidadeExibida;

    #[test]
    fn form_por_estado_e_nao_por_html_fixo() {
        let config = Configuracao {
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            opcoes: vec![
                Opcao {
                    coluna: Coluna::Atribuicao,
                    marcada: true,
                },
                Opcao {
                    coluna: Coluna::Anotacao,
                    marcada: false,
                },
            ],
        };
        let html = html(&config);
        assert!(html.contains("name='chkSinAtribuicao' class='infraCheckbox' checked"));
        assert!(html.contains("name='chkSinAnotacao' class='infraCheckbox'>"));
        assert!(html.contains("id='frmAcessoExternoCancelar'"));
        assert!(!html.contains("name='chkSinMarcadores'"));
    }
}
