---
type: CommandContract
title: Controle de prazo do processo
operation: processo.prazo
command: ies processo prazo NUP [--ate DD/MM/AAAA | --dias N | --excluir] [--confirmar]
effect: read-and-preview-write-process-state
implementation: partial
---

# `ies processo prazo NUP` (Refs #272)

Lê a situação do controle de prazo pela tela Web oferecida na barra do
processo. `--ate` e `--dias` produzem uma **prévia**; nenhuma escrita
é enviada enquanto o `--confirmar` não estiver coberto por POST único
e reconciliação de uma nova leitura. `--excluir` depende da lista de
prazos da unidade e também está bloqueado nesta fatia.

## Entrada e saída

- `NUP`: validado antes dos arquivos; uso inválido é saída 2.
- `--ate DD/MM/AAAA`: data certa, calendário gregoriano e ano positivo.
- `--dias N`: 1 a 999 dias corridos; a contagem é do SEI.
- `--excluir`: exclusivo com os dois anteriores.
- `--confirmar`: **recusado antes de qualquer rede** até a escrita ser
  segura e testada; não há POST nesta implementação.
- `--unidade`: conferida na abertura pela tela do processo.
- Saída JSON: `nup`, `id_procedimento`, `unidade`, `origem=web`,
  `atual` (`sem_prazo`, `data` ou `concluido`), `solicitado`,
  `campos_da_previa`, `confirmado=false` e `requisicoes`.

## Caminho e orçamento

1. Abrir o processo por `ies processo NUP`, com unidade exigida e
   sem andamentos/propriedades/marcadores. O número de GETs varia
   conforme o índice, como no contrato de abertura.
2. A barra do nó `PROCESSO` precisa publicar um único link assinado
   `controle_prazo_definir`, com `id_procedimento` igual ao aberto.
3. Mais **um GET** da tela assinada; nenhum POST em leitura ou prévia.
4. Ler exclusivamente o formulário `frmControlePrazoCadastro`, método
   POST, ação e id do processo conferidos, ocultos de identidade,
   `sbmDefinirControlePrazo`, `rdoPrazo` (data 1, dias 2,
   opcionalmente concluir 3), `txtPrazo`, `txtDias`,
   `chkSinDiasUteis`. Controle duplicado, desabilitado, alterado
   ou de outro processo: falha fechada. Atributos via
   `html_atributos`, nunca executar JavaScript.
5. A prévia para dias limpa `txtPrazo`, e a prévia por data limpa
   `txtDias`; não inclui `chkSinDiasUteis` sem pedido de úteis.

Orçamento esperado: **abertura + 1 GET** para `Ler`, `--ate`
ou `--dias`; **zero requisições** para uso inválido, `--excluir`
ainda não implementado ou `--confirmar`. Não há retry,
redirecionamento seguido nem restauração de tela.

## Falhas e subcódigos

| Subcódigo | Condição |
| --- | --- |
| `opcoes_de_prazo_incompativeis` | Combinação de opções mutuamente exclusivas |
| `data_do_prazo_invalida` | Data não existe |
| `dias_do_prazo_invalidos` | Contagem fora da faixa |
| `tela_de_prazo_nao_observada` | Formulário diferente da forma observada |
| `tela_de_prazo_de_outro_processo` | Identidade distinta |
| `barra_de_prazo_nao_interpretada` | Barra sem leitura segura |
| `links_de_prazo_repetidos` | Múltiplas ações de prazo |
| `endereco_de_prazo_nao_reconhecido` | URL fora da forma segura |
| `endereco_de_prazo_de_outro_processo` | URL de outro processo |
| `prazo_nao_publicado_na_barra` | Ação ausente |
| `tela_de_prazo_redirecionada` | GET não publicou HTML 200 |
| `sessao_encerrada_ao_ler_prazo` | Login no lugar da tela |
| `escrita_de_prazo_ainda_nao_disponivel` | Confirmação de escrita bloqueada |
| `exclusao_de_prazo_ainda_nao_disponivel` | Exclusão pela lista bloqueada |

Falhas do transporte e da abertura preservam os subcódigos específicos das
respectivas peças; não se publicam cookies nem URLs assinadas.

## Evidências e pendências

A forma Web está documentada em `Todos`
`tests/test_ciclo_prazo_010.py @323eb01` e nos índices imutáveis
`cassettes/unb/processo-prazo-unb--48614e412f28.md` e
`cassettes/go/processo-prazo-go--4d9b8462dae9.md`.
Aproveitam-se os cenários, não a arquitetura Python.

**Ainda falta para concluir #272:** implementação e testes integrados do
POST único, nova leitura e saída 3 para efeito incerto; `--excluir`
pela lista de prazos e tratamento de páginas, replay do acervo,
validação live no treinamento. Nunca escrever na produção.
