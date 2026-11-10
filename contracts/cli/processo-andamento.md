---
type: CommandContract
title: Registro de andamento no processo
operation: processo.andamento
command: ies processo andamento NUP --texto T [--confirmar]
effect: write-process-history
implementation: partial
---

# Registrar andamento no processo (#259)

O comando segue somente o link Web de Atualizar Andamento na barra do nó PROCESSO. A identidade do processo é opaca; URL assinada, cookie e segredo não aparecem na saída.

## Prévia e recusas

NUP e texto são validados antes de consultar a sessão. Texto sem conteúdo ou acima de 500 caracteres é recusado (saída 2, sem rede). O limite conta o texto integral, incluindo espaços e quebras de linha; o formulário e o resultado preservam o texto recebido. A leitura do processo obtém o histórico anterior e a barra, que deve oferecer a ação procedimento_atualizar_andamento. A tela deve publicar exatamente um formulário frmAtividadeListar com controles hdnIdAtividade, hdnIdProtocolo igual ao processo, txaDescricao e sbmSalvar. Campos de usuário imprevistos são recusados. Sem --confirmar, deriva o formulário mas não faz POST.

## Escrita e verificação

**`--confirmar` aprovado em treinamento (UnB, 10/11/2026) e liberado também em produção** ([relatório](../../docs/validacao-live/2026-11-10-processo-relacionar-andamento-unb.md)). Com --confirmar, envia um único POST no charset e action observados, sem seguir 302 e nunca repetir. Resposta esperada: 302 para procedimento_consultar_historico do mesmo processo. O 302 não prova escrita. Relê processo/histórico e exige visão igual, total_declarado acrescido de um, primeiro item com o texto normalizado por espaços e demais linhas antigas deslocadas em uma posição. A regra vale quando o texto já existia. Sem essa prova, saída 3 (efeito_incerto_no_andamento); não repetir sem conferência.

## Orçamento

Abertura do processo e histórico (variável conforme índice), mais 1 GET do formulário; com --confirmar soma 1 POST e nova leitura do processo/histórico. Sem consulta a marcadores ou propriedades. O comando informa as requisições realizadas na prévia e no sucesso.

## Evidência

[Cassette UnB](../../cassettes/unb/processo-andamento-registrar-unb--111df7e25e1e.md), [cassette GO](../../cassettes/go/processo-andamento-registrar-go--3d46a9421f08.md), [observação](../../docs/como-o-sei-funciona/registro-de-andamento.md), [decisão](../../docs/decisoes/processo-andamento.md).

**BDD-259-01:** teste `tests/aies_andamento_cli.rs` executa o binário `ies` real por HTTPS loopback contra o anti-IES e SQLite efêmero, exige a prévia com o texto original e confirma em releitura independente do SQLite que o histórico permaneceu intacto. É **PASS sintético**, sem POST. Inspeção via GitHub Actions verificou SHA-1/SHA-256 e estrutura dos cards UnB/GO (50 interações); isso **não constitui** confronto semântico independente, replay da CLI nem validação live. **BDD-259-02:** teste `tests/aies_andamento_cli.rs` exercita duas execuções da CLI real sobre HTTPS local e SQLite compartilhado: `--confirmar` envia um único POST, acrescenta um andamento e preserva os anteriores; a segunda CLI consulta `ies processo NUP --andamentos-completos` para conferir o registro. Prova **sintética**, sem confronto semântico independente com cassettes. **BDD-259-03:** o teste `tests/aies_andamento_cli.rs` mantém o POST único já persistido no SQLite e simula HTTP 503 somente na primeira leitura posterior; a CLI deve sair com código 3, orientar explicitamente a não repetir `--confirmar` e consultar o histórico. Uma segunda CLI independente verifica o resíduo no mesmo SQLite, sem nova escrita. É hipótese **sintética**, não um 503 observado na cassette ou no SEI. Confronto semântico com cassettes e treinamento externo: NOT_RUN nesta fatia.

## Subcódigos próprios

| Subcódigo | Quando ocorre |
| --- | --- |
| `atualizar_andamento_nao_oferecido` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `barra_do_andamento_fora_da_forma` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `campo_inesperado_no_formulario_do_andamento` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `charset_do_andamento_nao_suportado` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `controles_do_andamento_ausentes_ou_duplicados` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `efeito_incerto_no_andamento` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `endereco_de_atualizar_andamento_de_outro_processo` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `endereco_de_atualizar_andamento_nao_reconhecido` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `formulario_do_andamento_ausente_ou_duplicado` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `formulario_do_andamento_de_outro_processo` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `historico_nao_publicado_antes_do_andamento` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `links_de_atualizar_andamento_repetidos` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `sessao_encerrada_antes_do_andamento` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `tela_do_andamento_invalida` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `tela_do_andamento_redirecionada` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `unidade_da_sessao_nao_publicada` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
| `texto_do_andamento_vazio_ou_maior_que_500_caracteres` | Recusa nomeada na operação de andamento; preserve o subcódigo na saída. |
