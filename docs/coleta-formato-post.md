---
type: Guide
title: Roteiro de coleta do formato de POST de formulário do SEI
description: Captura por HAR do navegador e saneamento offline do formato de pesquisa e paginação, sem valores secretos
---

# Coleta do formato de POST de formulário

As cassettes públicas (`schema_version: 1`) registram cada POST apenas por
método, operação, parâmetro e caminho: **não há corpo nem cabeçalhos**.
Faltam, portanto, evidências do que a submissão de formulário contém.
Este roteiro descreve como um **operador autorizado**, em outra máquina e
numa instalação de **treinamento**, registra somente o **formato** desse
POST para três operações, sem que o IES emita qualquer requisição.

O roteiro não é teste live executado neste repositório, não autoriza login
automático nem prova que o IES reproduz o gesto. Complementa a
[coleta privada de cassettes](coleta-privada.md) e segue o
[método Contract-First](metodo-contract-first.md).

## Escopo

| Operação IES | `acao` do POST | Gesto único do operador | Orçamento sugerido |
| --- | --- | --- | --- |
| `processo.buscar` (prioridade) | `protocolo_pesquisar` | preencher filtro e clicar em Pesquisar | 3 |
| `documento.buscar` | `protocolo_pesquisar` | pesquisar documento pela mesma tela | 3 |
| `unidade.listar_processos` | `procedimento_controlar` | clicar uma vez na página seguinte do Controle | 2 |

A `acao` vem das cassettes registradas (por exemplo, a de
`documento-numero-e-busca`); a distinção entre pesquisa de processo e de
documento **não está demonstrada**: o formato coletado mostrará apenas os
campos efetivamente enviados. Nenhuma outra operação é aceita.

## O que é registrado

- Nomes dos campos do corpo, na **ordem enviada**, com o tipo declarado
  no HTML do formulário: `oculto`, `usuario`, `botao`, `misto` ou
  `desconhecido` (campo ausente no HTML, possivelmente gerado por JavaScript).
- Classe de cada valor: `vazio`, `literal` ou `redigido`. Só é `literal`
  o campo da allowlist (`hdnAcao`, `sbmPesquisar` ou `--literal-campo`)
  cujo valor tem até 32 letras, sem dígitos; qualquer outro é redigido.
- Campos declarados no formulário e não enviados (por exemplo, checkbox
  desmarcado), `method`, `enctype`, `accept-charset` e presença de `onsubmit`.
- Parâmetros da URL do POST por nome; apenas `acao`, `acao_origem` e
  `acao_retorno` podem ter valor literal. `infra_hash` é sempre redigido.
- Nomes de cabeçalhos, presença de `Origin` e `Referer`, se pertencem à
  instalação, a `acao` do Referer e o `Content-Type` (MIME e charset).
- Sequência de requisições de aplicação (método, `acao`, status), número
  de recursos estáticos, status e MIME da resposta, sem conteúdo.

**Nunca** são registrados valores de cookies, `infra_hash`, hashes,
números de processo ou documento, nomes, IDs, o rótulo da unidade ou o HTML.
Sequências de cinco ou mais dígitos em nomes de campo viram `{n}`.

O manifesto registra instalação, origem, versão SEI **declarada** no
arquivo de origem, operação, SHA-256 do rótulo de `lnkInfraUnidade`
(com prefixo fixo), orçamento, contagens, efeitos declarados e hashes.
O hash do rótulo só compara coletas; rótulos curtos podem ser adivinhados.

## Pré-requisitos

1. Autorização para **instalação + identidade + unidade + operação**,
   conforme a [política de acesso](interfaces/acesso-por-instalacao.md).
   Produção nunca é alvo, nem como substituto.
2. Orçamento declarado **antes** da captura e informado ao autorizador.
3. Arquivos privados `0600`, fora do repositório e sem links simbólicos:
   - **origem**: `https://host/caminho-base` e, opcional, `versao=X.Y.Z`,
     o mesmo formato do `IES_ORIGEM_FILE` da leitura da caixa;
   - **allowlist**: uma base por linha. Host sem marcador de treinamento
     (`treinamento`, `treina`, `homolog`, `hml`, `teste`) é recusado,
     mesmo listado, salvo se a linha terminar com `treinamento-declarado`;
   - **HAR**: exportado pelo próprio operador.

A sessão é a do navegador do operador: o IES não lê cookies, não faz
login e não envia requisições. Nada de senha, cookie ou URL assinada em
argumentos de processo.

## Passo a passo do operador

1. Abrir janela privada sem extensões e fazer login **manualmente** na
   instalação de treinamento. Captcha, 2FA ou tela inesperada: parar.
2. Navegar até a tela da operação **sem abrir processos**.
3. Abrir DevTools → Rede, ativar *Preserve log* e limpar o registro
   **imediatamente antes** do gesto. Recarregar a tela do formulário com
   o registro já ativo, para que o HTML entre no HAR (necessário aos tipos).
   Se o navegador pedir para **reenviar dados**, cancelar: seria outro POST.
4. Executar **um único** gesto da tabela. Não clicar em links de processo,
   documento ou ordenação; não trocar unidade.
5. Exportar o HAR com conteúdo das respostas (preferir a exportação
   saneada do navegador, quando houver) e aplicar `chmod 600`.
6. Executar o saneador:

```sh
python3 -I scripts/sanear_har_formulario.py \
  --har /privado/pesquisa.har \
  --origem-file /privado/origem --allowlist-file /privado/allowlist \
  --instalacao unb --operacao processo.buscar --orcamento 3 \
  --efeito filtro_lembrado_na_sessao --saida /privado/formato-unb-processo
```

7. Revisar `formato.json` e `manifesto.json` (modo `0600`, pasta `0700`).
   Publicar só após revisão humana; depois, **apagar o HAR**. O HAR
   contém segredos e dados reais e nunca vai para Git, issue ou cassette.

Efeitos aceitos em `--efeito`: `nenhum_observado`,
`filtro_lembrado_na_sessao`, `pagina_lembrada_na_sessao`,
`recebimento_registrado` e `nao_verificado`; o primeiro não combina
com os demais. Vocabulário fechado evita texto livre com dados reais.

## Avisos de efeito por operação

- **Pesquisa (processo ou documento):** o POST pode fazer o servidor
  lembrar filtros na sessão. Não clicar nos resultados: abrir um processo
  pode registrar recebimento ou visualização. O orçamento cobre recarregar
  o formulário, o POST e um eventual redirecionamento.
- **Paginação do Controle:** o POST pode gravar página, ordenação ou modo
  de visualização na sessão; ao terminar, voltar à primeira página
  manualmente e anotar o efeito. Não marcar checkboxes de processos.
- **Falha ou timeout:** não repetir automaticamente. Avaliar o efeito,
  registrar `FAIL` ou `NOT_RUN` e pedir nova autorização antes de recapturar.

Detalhes em [efeitos e verificação](como-o-sei-funciona/efeitos-e-verificacao.md),
[formulários](como-o-sei-funciona/formularios.md) e
[`infra_hash`](como-o-sei-funciona/infra-hash.md).

## Falhas fechadas

Qualquer código abaixo encerra com `FAIL | ... | codigo=...`, código 1,
sem criar a pasta de saída. O código é estável; o texto não classifica.

| Código | Condição verificável |
| --- | --- |
| `arquivo_privado_permissao_insegura` | HAR, origem ou allowlist com bits de grupo/outros |
| `arquivo_privado_link_simbolico` | arquivo de entrada é link simbólico |
| `origem_fora_da_allowlist` | base da origem não aparece na allowlist |
| `host_sem_marcador_de_treinamento` | host listado sem marcador nem declaração |
| `entrada_fora_da_origem` / `entrada_nao_https` | HAR inclui outro host, caminho-base ou esquema |
| `login_capturado_no_har` / `credencial_capturada_no_har` | HAR contém SIP, login ou campo de senha |
| `recurso_inesperado` / `metodo_inesperado` | entrada que não é `.php` nem estático, ou método diferente de GET/POST |
| `orcamento_excedido` | requisições de aplicação acima do orçamento |
| `post_alvo_ausente` | nenhum POST com a `acao` da operação |
| `corpo_multipart_nao_suportado` / `content_type_inesperado` | corpo não urlencoded |
| `nome_de_campo_inesperado` / `cabecalho_inesperado` | nome fora do alfabeto aceito |
| `vazamento_detectado` | valor redigido, hash ou sequência longa de dígitos na saída |
| `saida_existente` / `efeitos_contraditorios` | destino reutilizado ou efeitos incompatíveis |

`vazamento_detectado` é conservador: um valor redigido que coincide com
texto publicado (por exemplo, parte de uma `acao`) também bloqueia.

## Relato por instalação

| Instalação | `processo.buscar` | `documento.buscar` | `unidade.listar_processos` |
| --- | --- | --- | --- |
| UnB | NOT_RUN | NOT_RUN | NOT_RUN |
| UFRJ | NOT_RUN | NOT_RUN | NOT_RUN |
| Goiás | NOT_RUN | NOT_RUN | NOT_RUN |
| PEN1–PEN4 | NOT_RUN | NOT_RUN | NOT_RUN |

Motivo comum: neste repositório não há autorização, sessão nem acesso de
rede às instalações. O operador substitui cada célula por `PASS` (saída
saneada e revisada), `FAIL` (com o código) ou `NOT_RUN` (com o bloqueio).

## Autoteste offline

[`test_sanear_har.py`](../scripts/test_sanear_har.py) usa um HAR
**sintético**, gerado no próprio teste, e verifica a saída e as recusas.
Roda no hook `scripts-python` do [prek.toml](../prek.toml), o mesmo gate
do pré-push e do CI, ou diretamente:
`python3 -I -m unittest discover -s scripts -p "test_*.py"`.

## Limitações

- O HAR descreve o **navegador**, não o IES; a forma coletada é oráculo
  para testes RED, não prova de equivalência do transporte.
- Tipos vêm do HTML servido; campos alterados por JavaScript aparecem
  como `desconhecido` ou com tipo do HTML original.
- A classe `redigido` não diz se o valor é fixo, derivado da sessão ou
  digitado; comparar coletas de sessões distintas não revela isso sem valores.
- Uma coleta numa instalação não comprova as demais nem outra versão.
