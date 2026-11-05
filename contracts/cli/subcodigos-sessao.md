---
type: Contract
title: Subcódigos de sessão e transporte
description: Inventário literal complementar de falhas por contexto emissor
---

# Subcódigos de sessão e transporte

Este complemento enumera as condições de configuração, arquivos privados, login, status e transporte compartilhado pelos comandos Web.
As tabelas completam os contratos das operações; não mudam códigos de
saída, orçamento nem regras de recuperação. Delegações preservam o
subcódigo do leitor chamado. O mesmo literal pode aparecer em `subcodigo`
ou em uma falha aninhada; interpretar junto à operação e à etapa.
Ver [regras e contextos compartilhados](../../docs/erros-tipados.md).

Contratos de operação: [Login](sessao-login.md), [recusas HTTP](recusas-http.md) e [setup/status](../cli.md).

## Falhas tipadas

### arquivo_caixa

Emissor: [src/arquivo_caixa.rs](../../src/arquivo_caixa.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `arquivo_da_caixa_ja_existe` | Arquivo da caixa já existe. |
| `destino_da_caixa_sem_nome_de_arquivo` | Destino da caixa sem nome de arquivo. |
| `diretorio_do_arquivo_da_caixa_ausente` | Diretório do arquivo da caixa ausente. |
| `escrita_de_arquivo_da_caixa_falhou` | Escrita de arquivo da caixa falhou. |

### configuracao

Emissor: [src/configuracao.rs](../../src/configuracao.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `arquivo_configuracao_campo_invalido` | Arquivo configuração campo inválido. |
| `arquivo_configuracao_json_invalido` | Arquivo configuração JSON inválido. |
| `arquivo_configuracao_link_simbolico` | Arquivo configuração link simbólico. |
| `arquivo_configuracao_nao_pode_ser_lido` | Arquivo configuração não pode ser lido. |
| `arquivo_configuracao_permissoes_inseguras` | Arquivo configuração permissões inseguras. |
| `arquivo_privado_ausente` | Arquivo privado ausente. |
| `arquivo_privado_link_simbolico` | Arquivo privado link simbólico. |
| `arquivo_privado_nao_regular` | Arquivo privado não regular. |
| `arquivo_privado_permissoes_inseguras` | Arquivo privado permissões inseguras. |
| `arquivo_privado_vazio` | Arquivo privado vazio. |
| `caminho_cookie_nao_informado` | Caminho cookie não informado. |
| `caminho_origem_nao_informado` | Caminho origem não informado. |
| `caminho_privado_nao_absoluto` | Caminho privado não absoluto. |
| `caminho_url_controle_nao_informado` | Caminho URL controle não informado. |
| `metadados_arquivo_privado_nao_lidos` | Metadados arquivo privado não lidos. |

### sessao_login/erro

Emissor: [src/sessao_login/erro.rs](../../src/sessao_login/erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `arquivo_de_lock_do_login_inacessivel` | Arquivo de lock do login inacessível. |
| `botao_de_envio_do_login_nao_reconhecido` | Botão de envio do login não reconhecido. |
| `caminho_credenciais_nao_informado` | Caminho credenciais não informado. |
| `campo_das_credenciais_ausente` | Campo das credenciais ausente. |
| `campo_das_credenciais_repetido` | Campo das credenciais repetido. |
| `campo_das_credenciais_vazio` | Campo das credenciais vazio. |
| `captcha_exigido` | Captcha exigido. |
| `charset_da_tela_de_login_nao_suportado` | Charset da tela de login não suportado. |
| `controle_apos_o_login_nao_reconhecido` | Controle após o login não reconhecido. |
| `controle_web_nao_reconhecido` | Controle web não reconhecido. |
| `cookie_da_sessao_nao_recebido` | Cookie da sessão não recebido. |
| `cookie_recebido_fora_do_formato_aceito` | Cookie recebido fora do formato aceito. |
| `credencial_recusada` | Credencial recusada. |
| `destino_da_sessao_nao_eh_arquivo_regular` | Destino da sessão não é arquivo regular. |
| `diretorio_da_sessao_gravavel_por_outros` | Diretório real da sessão gravável por grupo ou outros, sem sticky bit (#211). |
| `sessao_trocada_durante_a_leitura` | O marcador de geração mudou duas vezes durante a leitura local de URL, origem e cookie (#222). |
| `gravacao_da_sessao_falhou` | Gravação da sessão falhou. |
| `linha_das_credenciais_sem_campo_conhecido` | Linha das credenciais sem campo conhecido. |
| `link_do_controle_de_processos_nao_reconhecido` | Link do controle de processos não reconhecido. |
| `localizacao_de_redirecionamento_nao_interpretavel` | Localização de redirecionamento não interpretável. |
| `login_em_andamento_em_outro_processo` | Login em andamento em outro processo. |
| `orgao_com_mais_de_uma_opcao_na_tela_de_login` | Órgão com mais de uma opção na tela de login. |
| `orgao_nao_encontrado_na_tela_de_login` | Órgão não encontrado na tela de login. |
| `redirecionamento_para_fora_da_instalacao` | Redirecionamento para fora da instalação. |
| `redirecionamento_que_repetiria_o_envio_do_login` | Redirecionamento que repetiria o envio do login. |
| `redirecionamento_do_login_sem_localizacao` | Redirecionamento sem localização. |
| `redirecionamentos_acima_do_limite` | Redirecionamentos acima do limite. |
| `regra_do_onsubmit_do_login_nao_observada` | Regra do onsubmit do login não observada. |
| `segundo_fator_exigido` | Segundo fator exigido. |
| `selecao_de_orgao_ausente_na_tela_de_login` | Seleção de órgão ausente na tela de login. |
| `selecao_de_orgao_repetida_na_tela_de_login` | Seleção de órgão repetida na tela de login. |
| `tela_apos_o_login_nao_reconhecida` | Tela após o login não reconhecida. |
| `tela_de_login_nao_reconhecida` | Tela de login não reconhecida. |
| `unidade_exibida_diferente_da_solicitada` | Unidade exibida diferente da solicitada. |

### status_web

Emissor: [src/status_web.rs](../../src/status_web.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `identidade_unidade_nao_observada` | Identidade unidade não observada. |

### transporte/falha

Emissor: [src/transporte/falha.rs](../../src/transporte/falha.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `arquivo_privado_acima_do_limite` | Arquivo privado acima do limite. |
| `arquivo_privado_ausente_ou_inacessivel` | Arquivo privado ausente ou inacessível. |
| `arquivo_privado_com_permissoes_inseguras` | Arquivo privado com permissões inseguras. |
| `arquivo_privado_nao_pode_ser_lido` | Arquivo privado não pode ser lido. |
| `bloqueio_de_seguranca_da_instalacao` | Bloqueio de segurança da instalação. |
| `caminho_nao_eh_arquivo` | Caminho não é arquivo. |
| `chave_de_consulta_do_controle_nao_canonica` | Chave de consulta do controle não canônica. |
| `cookie_invalido` | Cookie inválido. |
| `falha_de_rede` | Falha de rede. |
| `origem_nao_autorizada` | Origem não autorizada. |
| `pagina_de_login_recebida_no_controle` | Página de login recebida no controle. |
| `redirecionamento_recusado` | Redirecionamento recusado. |
| `resposta_excessiva` | Resposta excessiva. |
| `separador_de_consulta_do_controle_ambiguo` | Separador de consulta do controle ambíguo. |
| `sessao_nao_autorizada` | Sessão não autorizada. |
| `url_invalida` | URL inválida. |
