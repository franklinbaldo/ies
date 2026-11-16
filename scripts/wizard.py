#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.11"
# dependencies = ["keyring>=24.0.0", "rich>=13.0.0"]
# ///
"""Wizard do IES com Rich e cofre escopado por instalação."""
from __future__ import annotations

import getpass
from html.parser import HTMLParser
import json
import os
from pathlib import Path
import shutil
import ssl
import stat
import subprocess
import sys
import tomllib
import urllib.parse
import urllib.request

import keyring
from rich.console import Console
from rich.panel import Panel
from rich.prompt import Confirm, Prompt
from rich.table import Table

console = Console()
SERVICO_KEYRING = "ies"

INSTALACOES_PREDEFINIDAS = {
    "1": {"slug": "unb", "nome": "UnB 4.0.9 (Treinamento)", "origem": "https://sei.treinamento.unb.br/sei", "usuario": "aluno1", "senha": "aluno1", "orgao": "UnB"},
    "2": {"slug": "pen1", "nome": "PEN1 5.0.4 / Tramita (Homologação)", "origem": "https://sei.orgao1.tramita.processoeletronico.gov.br/sei", "usuario": "orgao1", "senha": "orgao1", "orgao": "ORGAO1"},
    "3": {"slug": "go", "nome": "Goiás 5.0.3 (Treinamento)", "origem": "https://seitreinamento.go.gov.br/sei", "usuario": "aluno05", "senha": "abc123456", "orgao": "SEAD"},
    "4": {"slug": "ufrj", "nome": "UFRJ (Treinamento)", "origem": "https://teste.sei.tic.ufrj.br/sei", "usuario": "servidor1", "senha": "servidor1", "orgao": "UFRJ"},
}

SEI_ENTRYPOINT_SEGMENTS = ("/sip/login.php", "/sei/controlador.php", "/sei/inicializar.php")


def chave_no_keyring(username: str, root_url: str) -> str:
    parsed = urllib.parse.urlparse(root_url)
    instancia = (parsed.netloc or parsed.path).strip("/").casefold()
    return f"{username}@{instancia}" if instancia else username


def servico_keyring(root_url: str) -> str:
    parsed = urllib.parse.urlparse(root_url)
    instancia = (parsed.netloc or parsed.path).strip("/").casefold()
    return f"{SERVICO_KEYRING}:{instancia}" if instancia else SERVICO_KEYRING


def obter_senha_keyring(username: str, root_url: str) -> str | None:
    chave = chave_no_keyring(username, root_url)
    servico = servico_keyring(root_url)
    candidatos = [
        (servico, username),
        (f"{username}.{servico}", username),
        (SERVICO_KEYRING, chave),
        (f"{chave}.{SERVICO_KEYRING}", chave),
    ]
    for s, u in candidatos:
        try:
            pwd = keyring.get_password(s, u)
            if pwd:
                return pwd
        except Exception:
            pass
    return None


def salvar_senha_keyring(username: str, root_url: str, senha: str) -> None:
    chave = chave_no_keyring(username, root_url)
    servico = servico_keyring(root_url)
    destinos = [
        (servico, username),
        (f"{username}.{servico}", username),
        (SERVICO_KEYRING, chave),
        (f"{chave}.{SERVICO_KEYRING}", chave),
    ]
    for s, u in destinos:
        try:
            keyring.set_password(s, u, senha)
        except Exception:
            pass


def carregar_config_toml(caminho: Path) -> dict | None:
    if not caminho.is_file():
        return None
    try:
        dados = tomllib.loads(caminho.read_text(encoding="utf-8"))
        if dados.get("sei_web_url") and dados.get("sei_usuario"):
            return dados
    except Exception:
        pass
    return None


def salvar_config_toml(caminho: Path, dados: dict) -> None:
    linhas = [
        f'sei_web_url = "{dados["sei_web_url"]}"',
        f'sei_usuario = "{dados["sei_usuario"]}"',
        f'sei_sigla_orgao = "{dados.get("sei_sigla_orgao", "")}"',
        f'sei_sigla_orgao_sistema = "{dados.get("sei_sigla_orgao_sistema", "")}"',
        f'sei_sigla_sistema = "{dados.get("sei_sigla_sistema", "SEI")}"',
        f'sei_verify_ssl = {str(dados.get("sei_verify_ssl", True)).lower()}',
    ]
    if dados.get("sei_unidade"):
        linhas.append(f'sei_unidade = "{dados["sei_unidade"]}"')
    caminho.parent.mkdir(parents=True, exist_ok=True)
    caminho.write_text("\n".join(linhas) + "\n", encoding="utf-8")


class SelOrgaoParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.in_sel = False
        self.options: list[tuple[str, str]] = []
        self.current_val = ""
        self.current_text = ""

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        d = {k.lower(): (v or "") for k, v in attrs}
        if tag.lower() == "select" and (d.get("name") == "selOrgao" or d.get("id") == "selOrgao"):
            self.in_sel = True
        elif self.in_sel and tag.lower() == "option":
            self.current_val, self.current_text = d.get("value", ""), ""

    def handle_endtag(self, tag: str) -> None:
        if tag.lower() == "select" and self.in_sel:
            self.in_sel = False
        elif self.in_sel and tag.lower() == "option":
            val, label = self.current_val.strip(), self.current_text.strip()
            if val and val != "null" and label:
                self.options.append((val, label))
            self.current_val = self.current_text = ""

    def handle_data(self, data: str) -> None:
        if self.in_sel:
            self.current_text += data


def normalizar_url_sei(raw_url: str) -> tuple[str, str]:
    parsed = urllib.parse.urlparse(raw_url.strip())
    if not parsed.scheme or not parsed.netloc:
        if raw_url.startswith("sei."):
            parsed = urllib.parse.urlparse("https://" + raw_url.strip())
        else:
            raise ValueError(f"URL inválida: '{raw_url}'. Deve começar com https://")

    path = parsed.path
    path_lower = path.lower()
    for seg in SEI_ENTRYPOINT_SEGMENTS:
        idx = path_lower.find(seg)
        if idx != -1:
            path = path[:idx]
            break

    p = path.rstrip("/")
    if p.endswith("/sei"):
        return f"{parsed.scheme}://{parsed.netloc}{p[:-4]}", f"{parsed.scheme}://{parsed.netloc}{p}"
    if not p or p == "/":
        return f"{parsed.scheme}://{parsed.netloc}", f"{parsed.scheme}://{parsed.netloc}/sei"
    return f"{parsed.scheme}://{parsed.netloc}{p}", f"{parsed.scheme}://{parsed.netloc}{p}/sei"


def sondar_instalacao_sei(url_alvo: str) -> tuple[str, list[tuple[str, str]]]:
    headers = {"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"}
    req = urllib.request.Request(url_alvo, headers=headers)
    ctx = ssl.create_default_context()
    try:
        with urllib.request.urlopen(req, timeout=12, context=ctx) as resp:
            final_url = resp.geturl()
            charset = resp.headers.get_content_charset() or "latin-1"
            html = resp.read().decode(charset, errors="replace")
    except Exception as exc:
        console.print(f"[yellow]  ! Aviso na sondagem: {exc}[/yellow]")
        return "", []

    query = urllib.parse.parse_qs(urllib.parse.urlparse(final_url).query)
    sigla_sip = query.get("sigla_orgao_sistema", [""])[0]
    parser = SelOrgaoParser()
    parser.feed(html)
    return sigla_sip, parser.options


def escolher_orgao(orgaos: list[tuple[str, str]]) -> str:
    if not orgaos:
        return Prompt.ask("[cyan]Sigla do Órgão (exatamente como em selOrgao)[/cyan]")
    if len(orgaos) == 1:
        id_val, rotulo = orgaos[0]
        console.print(f"[green]✓ Órgão único detectado:[/green] [bold]{rotulo}[/bold] (id {id_val})")
        return rotulo

    tabela = Table(title=f"Órgãos disponíveis nesta instalação ({len(orgaos)})", show_header=True, header_style="bold magenta")
    tabela.add_column("Nº", justify="right", style="cyan", width=4)
    tabela.add_column("Sigla / Nome", style="white")
    tabela.add_column("ID", justify="center", style="dim", width=6)
    for idx, (id_val, rotulo) in enumerate(orgaos, start=1):
        tabela.add_row(str(idx), rotulo, str(id_val))
    console.print(tabela)

    while True:
        entrada = Prompt.ask(f"[bold cyan]Escolha o órgão (1-{len(orgaos)}) ou digite a sigla[/bold cyan]").strip()
        if not entrada:
            continue
        if entrada.isdigit() and 1 <= int(entrada) <= len(orgaos):
            return orgaos[int(entrada) - 1][1]
        correspondencias = [r for _, r in orgaos if r.lower() == entrada.lower()]
        if len(correspondencias) == 1:
            return correspondencias[0]
        parciais = [r for _, r in orgaos if entrada.lower() in r.lower()]
        if len(parciais) == 1:
            return parciais[0]
        elif len(parciais) > 1:
            console.print(f"[yellow]Vários órgãos encontrados: {', '.join(parciais[:5])}...[/yellow]")
        else:
            console.print(f"[red]Nenhum órgão corresponde a '{entrada}'.[/red]")


def ajustar_permissoes_privadas(caminho: Path, eh_diretorio: bool = False) -> None:
    if os.name != "nt":
        modo = stat.S_IRWXU if eh_diretorio else (stat.S_IRUSR | stat.S_IWUSR)
        os.chmod(caminho, modo)


def encontrar_binario_ies() -> str:
    if candidato := shutil.which("ies"):
        return candidato
    home = Path.home()
    ext = ".exe" if os.name == "nt" else ""
    for loc in (home / ".local/bin" / f"ies{ext}", home / "bin" / f"ies{ext}", Path(f"target/release/ies{ext}")):
        if loc.is_file():
            return str(loc)
    return "ies"


def ler_senha(prompt: str = "Senha: ") -> str:
    try:
        if sys.stdin.isatty():
            return getpass.getpass(prompt)
    except Exception:
        pass
    return Prompt.ask(prompt, password=True)


def main() -> int:
    console.print(
        Panel.fit(
            "[bold white]IES — Assistente de Configuração[/bold white]\n"
            "[cyan]Descoberta automática & Cofre Seguro (OS Keyring)[/cyan]",
            border_style="bright_blue",
            padding=(1, 2),
        )
    )

    home = Path.home()
    caminho_toml_ies = home / ".ies" / "config.toml"
    caminho_toml_todos = home / ".todos" / "config.toml"
    config_existente = carregar_config_toml(caminho_toml_ies) or carregar_config_toml(caminho_toml_todos)

    tabela = Table(title="Opções de Instalação", show_header=True, header_style="bold green")
    tabela.add_column("Opção", justify="center", style="cyan", width=6)
    tabela.add_column("Instalação", style="bold white")
    tabela.add_column("Origem", style="dim")
    tabela.add_column("Cofre", justify="center", style="green")

    tem_existente = config_existente is not None
    if tem_existente:
        web_url = config_existente["sei_web_url"]
        user_ex = config_existente["sei_usuario"]
        orgao_ex = config_existente.get("sei_sigla_orgao", "")
        senha_keyring = obter_senha_keyring(user_ex, web_url)
        tem_senha_str = "[green]✓ gravada[/green]" if senha_keyring else "[yellow]pendente[/yellow]"
        tabela.add_row("0", f"Perfil TOML ({user_ex} @ {orgao_ex})", web_url, tem_senha_str)

    for chave, info in sorted(INSTALACOES_PREDEFINIDAS.items()):
        tabela.add_row(chave, info["nome"], info["origem"], "[dim]preset[/dim]")
    tabela.add_row("5", "Conectar ao meu SEI", "Personalizado (URL)", "[cyan]novo[/cyan]")
    console.print(tabela)

    padrao = "0" if tem_existente else "5"
    escolha = Prompt.ask("[bold]Opção desejada[/bold]", default=padrao)

    if escolha == "0" and tem_existente:
        web_url = config_existente["sei_web_url"]
        _, origem = normalizar_url_sei(web_url)
        usuario = config_existente["sei_usuario"]
        orgao = config_existente.get("sei_sigla_orgao", "")
        slug = f"{config_existente.get('sei_sigla_orgao_sistema', 'sei').lower()}-{orgao.lower()}"

        senha = obter_senha_keyring(usuario, web_url)
        if not senha:
            console.print(f"[yellow]Senha não encontrada no cofre '{SERVICO_KEYRING}'. Digite-a para salvar:[/yellow]")
            senha = ler_senha("Senha: ")
            salvar_senha_keyring(usuario, web_url, senha)
        else:
            console.print(f"[green]✓ Senha obtida com segurança do Keyring ({servico_keyring(web_url)}).[/green]")

    elif escolha in INSTALACOES_PREDEFINIDAS:
        cfg = INSTALACOES_PREDEFINIDAS[escolha]
        slug, origem, usuario, senha, orgao = cfg["slug"], cfg["origem"], cfg["usuario"], cfg["senha"], cfg["orgao"]
        web_url = origem[:-4] if origem.endswith("/sei") else origem
        salvar_senha_keyring(usuario, web_url, senha)

    elif escolha == "5":
        raw_url = Prompt.ask("\n[bold cyan]Cole a URL do seu SEI (ex: https://sei.sistemas.ro.gov.br)[/bold cyan]").strip()
        while not raw_url:
            raw_url = Prompt.ask("[bold cyan]Cole a URL do seu SEI[/bold cyan]").strip()

        try:
            web_url, origem = normalizar_url_sei(raw_url)
        except ValueError as err:
            console.print(f"[red]Erro:[/red] {err}")
            return 1

        console.print(f"  [dim]Origem do SEI:[/dim] [bold]{origem}[/bold]")
        with console.status("[bold green]Sondando login e catálogo de órgãos...[/bold green]", spinner="dots"):
            sigla_sip, orgaos = sondar_instalacao_sei(origem)

        if sigla_sip:
            console.print(f"  [dim]Ambiente SIP:[/dim]  [bold]{sigla_sip}[/bold]")

        orgao = escolher_orgao(orgaos)
        usuario = Prompt.ask("[bold cyan]Usuário (login)[/bold cyan]").strip()
        while not usuario:
            usuario = Prompt.ask("[bold cyan]Usuário[/bold cyan]").strip()

        senha_salva = obter_senha_keyring(usuario, web_url)
        serv_atual = servico_keyring(web_url)
        if senha_salva:
            if Confirm.ask(f"Senha encontrada no cofre ({serv_atual}). Usar?", default=True):
                senha = senha_salva
            else:
                senha = ler_senha("Nova Senha: ")
                salvar_senha_keyring(usuario, web_url, senha)
        else:
            senha = ler_senha("Senha: ")
            salvar_senha_keyring(usuario, web_url, senha)

        slug_sugerido = f"{sigla_sip.lower()}-{orgao.lower()}" if sigla_sip else orgao.lower()
        slug = Prompt.ask("[bold]Nome do perfil local[/bold]", default=slug_sugerido).strip()

        dados_toml = {
            "sei_web_url": web_url,
            "sei_usuario": usuario,
            "sei_sigla_orgao": orgao,
            "sei_sigla_orgao_sistema": sigla_sip,
            "sei_sigla_sistema": "SEI",
            "sei_verify_ssl": True,
        }
        salvar_config_toml(caminho_toml_ies, dados_toml)
        console.print(f"[green]✓ Configuração salva em:[/green] {caminho_toml_ies}")
        console.print(f"[green]✓ Senha gravada com segurança no cofre ({serv_atual}).[/green]")
    else:
        console.print(f"[red]Opção inválida:[/red] {escolha}")
        return 1

    diretorio_base = Path.home() / ".ies" / slug
    diretorio_base.mkdir(parents=True, exist_ok=True)
    ajustar_permissoes_privadas(diretorio_base, eh_diretorio=True)

    arquivo_origem = diretorio_base / "origem"
    arquivo_cookie = diretorio_base / "cookie"
    arquivo_url_controle = diretorio_base / "url-controle"
    arquivo_credenciais = diretorio_base / "credenciais"
    arquivo_config = diretorio_base / "ies-config.json"

    arquivo_origem.write_text(origem, encoding="utf-8")
    ajustar_permissoes_privadas(arquivo_origem)

    if not arquivo_cookie.exists():
        arquivo_cookie.write_text("", encoding="utf-8")
    ajustar_permissoes_privadas(arquivo_cookie)

    if not arquivo_url_controle.exists():
        arquivo_url_controle.write_text("", encoding="utf-8")
    ajustar_permissoes_privadas(arquivo_url_controle)

    conteudo_credenciais = f"usuario={usuario}\nsenha={senha}\norgao={orgao}\n"
    arquivo_credenciais.write_text(conteudo_credenciais, encoding="utf-8")
    ajustar_permissoes_privadas(arquivo_credenciais)

    if arquivo_config.exists():
        arquivo_config.unlink()

    binario_ies = encontrar_binario_ies()
    console.print("\n[cyan]Registrando arquivos privados com 'ies setup'...[/cyan]")

    cmd_setup = [
        binario_ies, "setup",
        "--config-file", str(arquivo_config.resolve()),
        "--url-controle-file", str(arquivo_url_controle.resolve()),
        "--cookie-file", str(arquivo_cookie.resolve()),
        "--origem-file", str(arquivo_origem.resolve()),
        "--json",
    ]

    try:
        subprocess.run(cmd_setup, capture_output=True, text=True, check=True)
        ajustar_permissoes_privadas(arquivo_config)
        console.print(f"[green]✓ Configuração gravada em:[/green] {arquivo_config}")
    except subprocess.CalledProcessError as erro:
        console.print(f"[red]Falha ao executar 'ies setup':\n{erro.stderr or erro.stdout}[/red]")
        return erro.returncode

    script_ps1 = diretorio_base / "ativar.ps1"
    script_ps1.write_text(
        f'$env:IES_CONFIG_FILE = "{arquivo_config.resolve()}"\n'
        f'if (Test-Path "{arquivo_credenciais.resolve()}") {{ $env:IES_CREDENCIAIS_FILE = "{arquivo_credenciais.resolve()}" }}\n'
        f'Write-Host "IES ativo: {slug}" -ForegroundColor Green\n',
        encoding="utf-8",
    )

    script_sh = diretorio_base / "ativar.sh"
    script_sh.write_text(
        f'export IES_CONFIG_FILE="{arquivo_config.resolve()}"\n'
        f'[ -f "{arquivo_credenciais.resolve()}" ] && export IES_CREDENCIAIS_FILE="{arquivo_credenciais.resolve()}"\n'
        f'echo "IES ativo: {slug}"\n',
        encoding="utf-8",
    )
    ajustar_permissoes_privadas(script_sh)

    console.print()
    if Confirm.ask("[bold]Deseja testar a conexão com 'ies login' agora?[/bold]", default=True):
        console.print("\n[cyan]Conectando e autenticando no SEI...[/cyan]")
        env = os.environ.copy()
        env["IES_CONFIG_FILE"] = str(arquivo_config.resolve())
        env["IES_CREDENCIAIS_FILE"] = str(arquivo_credenciais.resolve())

        proc_login = subprocess.run([binario_ies, "login", "--json"], capture_output=True, text=True, env=env)
        if proc_login.returncode == 0:
            try:
                dados = json.loads(proc_login.stdout)
                console.print("[bold green]✓ Login realizado com sucesso![/bold green]")
                console.print(f"  [dim]Instalação:[/dim]     {dados.get('instalacao')}")
                console.print(f"  [dim]Unidade ativa:[/dim]  [bold]{dados.get('unidade_exibida')}[/bold]")
                console.print(f"  [dim]Requisições:[/dim]    {dados.get('requisicoes')}")
            except json.JSONDecodeError:
                console.print(f"[green]✓ Resposta:[/green] {proc_login.stdout.strip()}")

            if Confirm.ask("\n[yellow]Deseja apagar a senha em texto claro do disco agora?[/yellow]", default=True):
                arquivo_credenciais.unlink(missing_ok=True)
                console.print("[green]✓ Arquivo 'credenciais' removido do disco. Sessão continua no cookie![/green]")
                console.print(f"  (A senha permanece protegida no Keyring [{servico_keyring(web_url)}] para logins futuros).")
        else:
            console.print(f"[red]Aviso no login (código {proc_login.returncode}):\n{proc_login.stderr or proc_login.stdout}[/red]")

    ativacao_cmd = f". {script_ps1.resolve()}" if os.name == "nt" else f"source {script_sh.resolve()}"
    painel_conteudo = (
        f"[bold white]Para ativar este perfil no terminal:[/bold white]\n\n"
        f"  [bold cyan]{ativacao_cmd}[/bold cyan]\n\n"
        f"[dim]Ou definindo as variáveis diretamente:[/dim]\n"
        f'  $env:IES_CONFIG_FILE = "{arquivo_config.resolve()}"\n'
    )
    if arquivo_credenciais.exists():
        painel_conteudo += f'  $env:IES_CREDENCIAIS_FILE = "{arquivo_credenciais.resolve()}"\n'

    painel_conteudo += (
        "\n[bold white]Comandos de exemplo:[/bold white]\n"
        "  [green]ies status[/green]\n"
        "  [green]ies unidade atual[/green]\n"
        "  [green]ies unidade recebidos[/green]\n"
        "  [green]ies processo <NUP>[/green]"
    )

    console.print(Panel(painel_conteudo, title=f"Perfil: {slug}", border_style="green"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
